use anyhow::{Result, bail};
use webauthn_rs::prelude::*;
use crate::database::DbPool;
use crate::app::services::user_service::UserService;
use crate::app::models::mfa_webauthn::{
    MfaWebAuthnCredential, NewMfaWebAuthnCredential, MfaWebAuthnChallenge, NewMfaWebAuthnChallenge
};
use crate::app::models::DieselUlid;
use diesel::prelude::*;
use serde_json;

const CHALLENGE_EXPIRY_MINUTES: i64 = 5;

pub struct MfaWebAuthnService {
    webauthn: Webauthn,
}

impl MfaWebAuthnService {
    /// Create new WebAuthn service instance
    pub fn new(rp_origin: &str, rp_id: &str, rp_name: &str) -> Result<Self> {
        let rp_origin = Url::parse(rp_origin)?;

        let builder = WebauthnBuilder::new(rp_id, &rp_origin)?;
        let webauthn = builder
            .rp_name(rp_name)
            .build()?;

        Ok(Self { webauthn })
    }

    /// Start WebAuthn registration for a user
    pub async fn start_registration(
        &self,
        pool: &DbPool,
        user_id: String,
        device_name: Option<String>,
    ) -> Result<CreationChallengeResponse> {
        // Get user
        let user = UserService::find_by_id(pool, user_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Get existing credentials for this user
        let existing_credentials = Self::get_user_credentials(pool, &user_id)?;

        let exclude_credentials: Vec<CredentialID> = existing_credentials
            .iter()
            .filter_map(|cred| serde_json::from_str::<Vec<u8>>(&cred.credential_id).ok())
            .map(|bytes| CredentialID::from(bytes))
            .collect();

        // Create user unique ID
        let user_unique_id = Uuid::parse_str(&user_id)
            .unwrap_or_else(|_| Uuid::new_v4());

        // Start registration
        let (creation_challenge_response, registration_state) = self.webauthn
            .start_passkey_registration(
                user_unique_id,
                &user.email,
                &user.name,
                Some(exclude_credentials),
            )?;

        // Store challenge - use JSON for serialization
        let challenge_json = serde_json::to_string(&registration_state)?;

        let new_challenge = MfaWebAuthnChallenge::new(
            user_id_ulid,
            challenge_json,
            "registration".to_string(),
            CHALLENGE_EXPIRY_MINUTES,
        );

        let mut conn = pool.get()?;
        diesel::insert_into(crate::schema::mfa_webauthn_challenges::table)
            .values(&new_challenge)
            .execute(&mut conn)?;

        tracing::info!(
            user_id = %user_id,
            "WebAuthn registration started"
        );

        Ok(creation_challenge_response)
    }

    /// Finish WebAuthn registration
    pub async fn finish_registration(
        &self,
        pool: &DbPool,
        user_id: String,
        credential: RegisterPublicKeyCredential,
        device_name: Option<String>,
    ) -> Result<()> {
        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Get and validate challenge
        let challenge = Self::get_challenge(pool, &user_id, "registration").await?;

        if challenge.is_expired() || challenge.is_used {
            bail!("Challenge has expired or been used");
        }

        // Parse registration state from JSON
        let registration_state: PasskeyRegistration = serde_json::from_str(&challenge.challenge)?;

        // Finish registration
        let passkey = self.webauthn
            .finish_passkey_registration(&credential, &registration_state)?;

        // Store credential - serialize the whole credential ID
        let credential_id_value = serde_json::to_string(passkey.cred_id())?;

        // Serialize passkey to JSON
        let public_key = serde_json::to_string(&passkey)?;

        let transports: Option<Vec<Option<String>>> = credential
            .response
            .transports
            .as_ref()
            .map(|t| t.iter().map(|s| Some(format!("{:?}", s))).collect());

        let new_credential = NewMfaWebAuthnCredential {
            id: DieselUlid::new(),
            user_id: user_id_ulid,
            credential_id: credential_id_value,
            public_key,
            counter: 0, // Start at 0, will be updated on authentication
            device_name,
            aaguid: None,
            transports,
            attestation_format: None,
            is_backup_eligible: false, // Default values
            is_backup_state: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut conn = pool.get()?;

        // Mark challenge as used
        diesel::update(crate::schema::mfa_webauthn_challenges::table.find(&challenge.id))
            .set(crate::schema::mfa_webauthn_challenges::is_used.eq(true))
            .execute(&mut conn)?;

        // Insert credential
        diesel::insert_into(crate::schema::mfa_webauthn_credentials::table)
            .values(&new_credential)
            .execute(&mut conn)?;

        // Update mfa_methods table
        diesel::insert_into(crate::schema::mfa_methods::table)
            .values((
                crate::schema::mfa_methods::id.eq(DieselUlid::new()),
                crate::schema::mfa_methods::user_id.eq(&user_id_ulid),
                crate::schema::mfa_methods::method_type.eq("webauthn"),
                crate::schema::mfa_methods::is_enabled.eq(true),
                crate::schema::mfa_methods::is_verified.eq(true),
                crate::schema::mfa_methods::created_at.eq(chrono::Utc::now()),
                crate::schema::mfa_methods::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)?;

        tracing::info!(
            user_id = %user_id,
            "WebAuthn credential registered successfully"
        );

        Ok(())
    }

    /// Start WebAuthn authentication
    pub async fn start_authentication(
        &self,
        pool: &DbPool,
        user_id: String,
    ) -> Result<RequestChallengeResponse> {
        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Get user's credentials
        let credentials = Self::get_user_credentials(pool, &user_id)?;

        if credentials.is_empty() {
            bail!("No WebAuthn credentials found for user");
        }

        // Convert to Passkey objects
        let passkeys: Vec<Passkey> = credentials
            .iter()
            .filter_map(|cred| serde_json::from_str(&cred.public_key).ok())
            .collect();

        if passkeys.is_empty() {
            bail!("No valid WebAuthn credentials found");
        }

        // Start authentication
        let (request_challenge_response, authentication_state) = self.webauthn
            .start_passkey_authentication(&passkeys)?;

        // Store challenge as JSON
        let challenge_json = serde_json::to_string(&authentication_state)?;

        let new_challenge = MfaWebAuthnChallenge::new(
            user_id_ulid,
            challenge_json,
            "authentication".to_string(),
            CHALLENGE_EXPIRY_MINUTES,
        );

        let mut conn = pool.get()?;
        diesel::insert_into(crate::schema::mfa_webauthn_challenges::table)
            .values(&new_challenge)
            .execute(&mut conn)?;

        tracing::info!(
            user_id = %user_id,
            "WebAuthn authentication started"
        );

        Ok(request_challenge_response)
    }

    /// Finish WebAuthn authentication
    pub async fn finish_authentication(
        &self,
        pool: &DbPool,
        user_id: String,
        credential: PublicKeyCredential,
    ) -> Result<bool> {
        let user_id_ulid = DieselUlid::from_string(&user_id)?;

        // Get and validate challenge
        let challenge = Self::get_challenge(pool, &user_id, "authentication").await?;

        if challenge.is_expired() || challenge.is_used {
            bail!("Challenge has expired or been used");
        }

        // Parse authentication state from JSON
        let authentication_state: PasskeyAuthentication = serde_json::from_str(&challenge.challenge)?;

        // Finish authentication
        let auth_result = self.webauthn
            .finish_passkey_authentication(&credential, &authentication_state)?;

        // Update credential counter
        let credential_id_value = serde_json::to_string(auth_result.cred_id())?;
        let new_counter = auth_result.counter();

        let mut conn = pool.get()?;

        // Mark challenge as used
        diesel::update(crate::schema::mfa_webauthn_challenges::table.find(&challenge.id))
            .set(crate::schema::mfa_webauthn_challenges::is_used.eq(true))
            .execute(&mut conn)?;

        // Update credential counter and last used
        diesel::update(crate::schema::mfa_webauthn_credentials::table)
            .filter(crate::schema::mfa_webauthn_credentials::user_id.eq(&user_id_ulid))
            .filter(crate::schema::mfa_webauthn_credentials::credential_id.eq(&credential_id_value))
            .set((
                crate::schema::mfa_webauthn_credentials::counter.eq(new_counter as i64),
                crate::schema::mfa_webauthn_credentials::last_used_at.eq(Some(chrono::Utc::now())),
                crate::schema::mfa_webauthn_credentials::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)?;

        tracing::info!(
            user_id = %user_id,
            "WebAuthn authentication successful"
        );

        Ok(true)
    }

    /// Get user's WebAuthn credentials
    fn get_user_credentials(pool: &DbPool, user_id_param: &str) -> Result<Vec<MfaWebAuthnCredential>> {
        use crate::schema::mfa_webauthn_credentials::dsl::*;

        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let mut conn = pool.get()?;

        let credentials = mfa_webauthn_credentials
            .filter(user_id.eq(&user_id_ulid))
            .filter(deleted_at.is_null())
            .select(MfaWebAuthnCredential::as_select())
            .load::<MfaWebAuthnCredential>(&mut conn)?;

        Ok(credentials)
    }

    /// Get stored challenge for user
    async fn get_challenge(
        pool: &DbPool,
        user_id_param: &str,
        challenge_type_filter: &str,
    ) -> Result<MfaWebAuthnChallenge> {
        let user_id_ulid = DieselUlid::from_string(user_id_param)?;
        let mut conn = pool.get()?;

        let result = crate::schema::mfa_webauthn_challenges::table
            .filter(crate::schema::mfa_webauthn_challenges::user_id.eq(&user_id_ulid))
            .filter(crate::schema::mfa_webauthn_challenges::challenge_type.eq(challenge_type_filter))
            .filter(crate::schema::mfa_webauthn_challenges::is_used.eq(false))
            .order(crate::schema::mfa_webauthn_challenges::created_at.desc())
            .select(MfaWebAuthnChallenge::as_select())
            .first::<MfaWebAuthnChallenge>(&mut conn)?;

        Ok(result)
    }

    /// Delete a WebAuthn credential
    pub async fn delete_credential(pool: &DbPool, credential_id_param: &str) -> Result<()> {
        use crate::schema::mfa_webauthn_credentials::dsl::*;

        let credential_ulid = DieselUlid::from_string(credential_id_param)?;
        let mut conn = pool.get()?;

        diesel::update(mfa_webauthn_credentials)
            .filter(id.eq(&credential_ulid))
            .set(deleted_at.eq(Some(chrono::Utc::now())))
            .execute(&mut conn)?;

        tracing::info!(
            credential_id = credential_id_param,
            "WebAuthn credential deleted"
        );

        Ok(())
    }

    /// List user's credentials
    pub async fn list_credentials(pool: &DbPool, user_id: String) -> Result<Vec<MfaWebAuthnCredential>> {
        Self::get_user_credentials(pool, &user_id)
    }
}
