use anyhow::{Result, bail};
use webauthn_rs::prelude::*;
use crate::database::DbPool;
use crate::app::services::user_service::UserService;
use crate::app::models::mfa_biometric::{MfaBiometricCredential, NewMfaBiometricCredential};
use crate::app::models::mfa_webauthn::{MfaWebAuthnChallenge, NewMfaWebAuthnChallenge};
use crate::app::models::DieselUlid;
use diesel::prelude::*;

const CHALLENGE_EXPIRY_MINUTES: i64 = 5;

pub struct MfaBiometricService {
    webauthn: Webauthn,
}

impl MfaBiometricService {
    /// Create new Biometric service instance
    pub fn new(rp_origin: &str, rp_id: &str, rp_name: &str) -> Result<Self> {
        let rp_origin = Url::parse(rp_origin)?;

        let builder = WebauthnBuilder::new(rp_id, &rp_origin)?;
        let webauthn = builder
            .rp_name(rp_name)
            .build()?;

        Ok(Self { webauthn })
    }

    /// Start biometric registration
    pub async fn start_registration(
        &self,
        pool: &DbPool,
        user_id: String,
        biometric_type: String,
        platform: String,
        device_name: Option<String>,
    ) -> Result<CreationChallengeResponse> {
        // Validate biometric type
        let valid_types = vec!["fingerprint", "face", "iris", "voice"];
        if !valid_types.contains(&biometric_type.as_str()) {
            bail!("Invalid biometric type. Must be one of: fingerprint, face, iris, voice");
        }

        // Get user
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Get existing credentials for this user
        let existing_credentials = Self::get_user_credentials(pool, &user_id)?;

        let exclude_credentials: Vec<CredentialID> = existing_credentials
            .iter()
            .filter_map(|cred| base64::decode(&cred.credential_id).ok())
            .map(|bytes| CredentialID::from(bytes))
            .collect();

        // Create user unique ID
        let user_unique_id = Uuid::parse_str(&user_id)
            .unwrap_or_else(|_| Uuid::new_v4());

        // Start registration with platform authenticator preference
        let (creation_challenge_response, registration_state) = self.webauthn
            .start_passkey_registration(
                user_unique_id,
                &user.email,
                &user.name,
                Some(exclude_credentials),
            )?;

        // Store challenge (reuse webauthn_challenges table)
        let challenge_json = serde_json::to_string(&registration_state)?;
        let metadata = serde_json::json!({
            "biometric_type": biometric_type,
            "platform": platform,
        });

        let new_challenge = MfaWebAuthnChallenge::new(
            user_id_ulid,
            challenge_json,
            "biometric_registration".to_string(),
            CHALLENGE_EXPIRY_MINUTES,
        );

        let mut conn = pool.get()?;
        diesel::insert_into(crate::schema::mfa_webauthn_challenges::table)
            .values(&new_challenge)
            .execute(&mut conn)?;

        tracing::info!(
            user_id = user_id,
            biometric_type = biometric_type,
            platform = platform,
            "Biometric registration started"
        );

        Ok(creation_challenge_response)
    }

    /// Finish biometric registration
    pub async fn finish_registration(
        &self,
        pool: &DbPool,
        user_id: String,
        credential: RegisterPublicKeyCredential,
        biometric_type: String,
        platform: String,
        device_name: Option<String>,
        device_id: Option<String>,
    ) -> Result<()> {
        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let device_id_ulid = device_id
            .map(|id| DieselUlid::from_string(&id))
            .transpose()?;

        // Get and validate challenge
        let challenge = Self::get_challenge(pool, &user_id, "biometric_registration").await?;

        if challenge.is_expired() || challenge.is_used {
            bail!("Challenge has expired or been used");
        }

        // Parse registration state
        let registration_state: PasskeyRegistration = serde_json::from_str(&challenge.challenge)?;

        // Finish registration
        let passkey = self.webauthn
            .finish_passkey_registration(&credential, &registration_state)?;

        // Store credential
        let credential_id = base64::encode(&passkey.cred_id().0);
        let public_key = serde_json::to_string(&passkey)?;

        let new_credential = NewMfaBiometricCredential {
            id: DieselUlid::new(),
            user_id: user_id_ulid,
            device_id: device_id_ulid,
            biometric_type: biometric_type.clone(),
            credential_id,
            public_key,
            platform: platform.clone(),
            device_name,
            is_platform_authenticator: true,
            counter: passkey.counter() as i64,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut conn = pool.get()?;

        // Mark challenge as used
        diesel::update(crate::schema::mfa_webauthn_challenges::table.find(&challenge.id))
            .set(crate::schema::mfa_webauthn_challenges::is_used.eq(true))
            .execute(&mut conn)?;

        // Insert credential
        diesel::insert_into(crate::schema::mfa_biometric_credentials::table)
            .values(&new_credential)
            .execute(&mut conn)?;

        // Update mfa_methods table
        use crate::schema::mfa_methods::dsl::*;

        let metadata = serde_json::json!({
            "biometric_type": biometric_type,
            "platform": platform,
        });

        diesel::insert_into(mfa_methods)
            .values((
                crate::schema::mfa_methods::id.eq(DieselUlid::new()),
                crate::schema::mfa_methods::user_id.eq(&user_id_ulid),
                crate::schema::mfa_methods::method_type.eq("biometric"),
                crate::schema::mfa_methods::is_enabled.eq(true),
                crate::schema::mfa_methods::is_verified.eq(true),
                crate::schema::mfa_methods::metadata.eq(Some(metadata)),
                crate::schema::mfa_methods::created_at.eq(chrono::Utc::now()),
                crate::schema::mfa_methods::updated_at.eq(chrono::Utc::now()),
            ))
            .on_conflict((crate::schema::mfa_methods::user_id, crate::schema::mfa_methods::method_type))
            .do_update()
            .set(crate::schema::mfa_methods::is_enabled.eq(true))
            .execute(&mut conn)?;

        tracing::info!(
            user_id = user_id,
            biometric_type = biometric_type,
            "Biometric credential registered successfully"
        );

        Ok(())
    }

    /// Start biometric authentication
    pub async fn start_authentication(
        &self,
        pool: &DbPool,
        user_id: String,
    ) -> Result<RequestChallengeResponse> {
        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Get user's credentials
        let credentials = Self::get_user_credentials(pool, &user_id)?;

        if credentials.is_empty() {
            bail!("No biometric credentials found for user");
        }

        // Convert to Passkey objects
        let passkeys: Vec<Passkey> = credentials
            .iter()
            .filter_map(|cred| serde_json::from_str(&cred.public_key).ok())
            .collect();

        if passkeys.is_empty() {
            bail!("No valid biometric credentials found");
        }

        // Start authentication
        let (request_challenge_response, authentication_state) = self.webauthn
            .start_passkey_authentication(&passkeys)?;

        // Store challenge
        let challenge_json = serde_json::to_string(&authentication_state)?;
        let new_challenge = MfaWebAuthnChallenge::new(
            user_id_ulid,
            challenge_json,
            "biometric_authentication".to_string(),
            CHALLENGE_EXPIRY_MINUTES,
        );

        let mut conn = pool.get()?;
        diesel::insert_into(crate::schema::mfa_webauthn_challenges::table)
            .values(&new_challenge)
            .execute(&mut conn)?;

        tracing::info!(
            user_id = user_id,
            "Biometric authentication started"
        );

        Ok(request_challenge_response)
    }

    /// Finish biometric authentication
    pub async fn finish_authentication(
        &self,
        pool: &DbPool,
        user_id: String,
        credential: PublicKeyCredential,
    ) -> Result<bool> {
        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Get and validate challenge
        let challenge = Self::get_challenge(pool, &user_id, "biometric_authentication").await?;

        if challenge.is_expired() || challenge.is_used {
            bail!("Challenge has expired or been used");
        }

        // Parse authentication state
        let authentication_state: PasskeyAuthentication = serde_json::from_str(&challenge.challenge)?;

        // Finish authentication
        let auth_result = self.webauthn
            .finish_passkey_authentication(&credential, &authentication_state)?;

        // Update credential counter
        let credential_id = base64::encode(&auth_result.cred_id().0);
        let new_counter = auth_result.counter() as i64;

        let mut conn = pool.get()?;

        // Mark challenge as used
        diesel::update(crate::schema::mfa_webauthn_challenges::table.find(&challenge.id))
            .set(crate::schema::mfa_webauthn_challenges::is_used.eq(true))
            .execute(&mut conn)?;

        // Update credential counter and last used
        use crate::schema::mfa_biometric_credentials::dsl::*;

        diesel::update(mfa_biometric_credentials)
            .filter(user_id.eq(&user_id_ulid))
            .filter(credential_id.eq(&credential_id))
            .set((
                counter.eq(new_counter),
                last_used_at.eq(Some(chrono::Utc::now())),
                updated_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)?;

        tracing::info!(
            user_id = user_id,
            "Biometric authentication successful"
        );

        Ok(true)
    }

    /// Get user's biometric credentials
    fn get_user_credentials(pool: &DbPool, user_id_param: &str) -> Result<Vec<MfaBiometricCredential>> {
        use crate::schema::mfa_biometric_credentials::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let mut conn = pool.get()?;

        let credentials = mfa_biometric_credentials
            .filter(user_id.eq(&user_id_ulid))
            .filter(deleted_at.is_null())
            .select(MfaBiometricCredential::as_select())
            .load::<MfaBiometricCredential>(&mut conn)?;

        Ok(credentials)
    }

    /// Get stored challenge for user
    async fn get_challenge(
        pool: &DbPool,
        user_id_param: &str,
        challenge_type_filter: &str,
    ) -> Result<MfaWebAuthnChallenge> {
        use crate::schema::mfa_webauthn_challenges::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let mut conn = pool.get()?;

        let challenge = mfa_webauthn_challenges
            .filter(user_id.eq(&user_id_ulid))
            .filter(challenge_type.eq(challenge_type_filter))
            .filter(is_used.eq(false))
            .order(created_at.desc())
            .select(MfaWebAuthnChallenge::as_select())
            .first::<MfaWebAuthnChallenge>(&mut conn)?;

        Ok(challenge)
    }

    /// Delete a biometric credential
    pub async fn delete_credential(pool: &DbPool, credential_id_param: &str) -> Result<()> {
        use crate::schema::mfa_biometric_credentials::dsl::*;

        let credential_ulid = DieselUlid::from_string(credential_id_param)?;
        let mut conn = pool.get()?;

        diesel::update(mfa_biometric_credentials)
            .filter(id.eq(&credential_ulid))
            .set(deleted_at.eq(Some(chrono::Utc::now())))
            .execute(&mut conn)?;

        tracing::info!(
            credential_id = credential_id_param,
            "Biometric credential deleted"
        );

        Ok(())
    }

    /// List user's biometric credentials
    pub async fn list_credentials(pool: &DbPool, user_id: String) -> Result<Vec<MfaBiometricCredential>> {
        Self::get_user_credentials(pool, &user_id)
    }

    /// Get credentials by biometric type
    pub async fn list_by_type(
        pool: &DbPool,
        user_id: String,
        bio_type: String,
    ) -> Result<Vec<MfaBiometricCredential>> {
        use crate::schema::mfa_biometric_credentials::dsl::*;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;
        let mut conn = pool.get()?;

        let credentials = mfa_biometric_credentials
            .filter(user_id.eq(&user_id_ulid))
            .filter(biometric_type.eq(&bio_type))
            .filter(deleted_at.is_null())
            .select(MfaBiometricCredential::as_select())
            .load::<MfaBiometricCredential>(&mut conn)?;

        Ok(credentials)
    }
}
