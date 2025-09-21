// @generated automatically by Diesel CLI.

diesel::table! {
    cities (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        province_id -> Bpchar,
        name -> Varchar,
        code -> Nullable<Varchar>,
        latitude -> Nullable<Numeric>,
        longitude -> Nullable<Numeric>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    countries (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        iso_code -> Varchar,
        phone_code -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    events (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 255]
        event_name -> Varchar,
        event_data -> Jsonb,
        #[max_length = 255]
        aggregate_id -> Nullable<Varchar>,
        #[max_length = 255]
        aggregate_type -> Nullable<Varchar>,
        version -> Nullable<Int4>,
        occurred_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    organization_position_levels (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        code -> Nullable<Varchar>,
        level -> Int4,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    organization_positions (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        code -> Nullable<Varchar>,
        #[max_length = 26]
        organization_position_level_id -> Bpchar,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    jobs (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 255]
        queue_name -> Varchar,
        #[max_length = 255]
        job_name -> Varchar,
        payload -> Jsonb,
        attempts -> Int4,
        max_attempts -> Int4,
        #[max_length = 50]
        status -> Varchar,
        priority -> Int4,
        available_at -> Timestamptz,
        reserved_at -> Nullable<Timestamptz>,
        processed_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        error_message -> Nullable<Text>,
        timeout_seconds -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    migrations (id) {
        id -> Int4,
        #[max_length = 255]
        migration -> Varchar,
        batch -> Int4,
        executed_at -> Timestamptz,
    }
}

diesel::table! {
    notifications (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[sql_name = "type"]
        #[max_length = 255]
        type_ -> Varchar,
        #[max_length = 255]
        notifiable_type -> Varchar,
        #[max_length = 255]
        notifiable_id -> Varchar,
        data -> Jsonb,
        channels -> Array<Nullable<Text>>,
        read_at -> Nullable<Timestamptz>,
        sent_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        retry_count -> Nullable<Int4>,
        max_retries -> Nullable<Int4>,
        error_message -> Nullable<Text>,
        priority -> Nullable<Int4>,
        scheduled_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_access_tokens (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Nullable<Bpchar>,
        #[max_length = 26]
        client_id -> Bpchar,
        name -> Nullable<Varchar>,
        scopes -> Nullable<Text>,
        revoked -> Bool,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_auth_codes (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 26]
        client_id -> Bpchar,
        scopes -> Nullable<Text>,
        revoked -> Bool,
        expires_at -> Nullable<Timestamptz>,
        challenge -> Nullable<Varchar>,
        challenge_method -> Nullable<Varchar>,
        redirect_uri -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_clients (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Nullable<Bpchar>,
        name -> Varchar,
        secret -> Nullable<Varchar>,
        provider -> Nullable<Varchar>,
        redirect_uris -> Text,
        personal_access_client -> Bool,
        password_client -> Bool,
        revoked -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_personal_access_clients (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        client_id -> Bpchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_refresh_tokens (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        access_token_id -> Bpchar,
        revoked -> Bool,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_scopes (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        description -> Nullable<Text>,
        is_default -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    organizations (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        #[max_length = 26]
        parent_id -> Nullable<Bpchar>,
        code -> Nullable<Varchar>,
        description -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    provinces (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        country_id -> Bpchar,
        name -> Varchar,
        code -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    push_subscriptions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        endpoint -> Text,
        p256dh_key -> Text,
        auth_key -> Text,
        user_agent -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sys_model_has_permissions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 255]
        model_type -> Varchar,
        #[max_length = 26]
        model_id -> Bpchar,
        #[max_length = 26]
        permission_id -> Bpchar,
        #[max_length = 255]
        scope_type -> Nullable<Varchar>,
        #[max_length = 26]
        scope_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sys_model_has_roles (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 255]
        model_type -> Varchar,
        #[max_length = 26]
        model_id -> Bpchar,
        #[max_length = 26]
        role_id -> Bpchar,
        #[max_length = 255]
        scope_type -> Nullable<Varchar>,
        #[max_length = 26]
        scope_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sys_permissions (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        guard_name -> Varchar,
        resource -> Nullable<Varchar>,
        action -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sys_roles (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        description -> Nullable<Text>,
        guard_name -> Varchar,
        #[max_length = 255]
        scope_type -> Nullable<Varchar>,
        #[max_length = 26]
        scope_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sys_users (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        email -> Varchar,
        email_verified_at -> Nullable<Timestamptz>,
        username -> Nullable<Varchar>,
        password -> Varchar,
        remember_token -> Nullable<Varchar>,
        password_reset_token -> Nullable<Varchar>,
        password_reset_expires_at -> Nullable<Timestamptz>,
        refresh_token -> Nullable<Varchar>,
        refresh_token_expires_at -> Nullable<Timestamptz>,
        avatar -> Nullable<Varchar>,
        birthdate -> Nullable<Date>,
        failed_login_attempts -> Int4,
        google_id -> Nullable<Varchar>,
        last_login_at -> Nullable<Timestamptz>,
        last_seen_at -> Timestamptz,
        locale -> Nullable<Varchar>,
        locked_until -> Nullable<Timestamptz>,
        phone_number -> Nullable<Varchar>,
        phone_verified_at -> Nullable<Timestamptz>,
        zoneinfo -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_organizations (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 26]
        organization_id -> Bpchar,
        #[max_length = 26]
        organization_position_id -> Bpchar,
        is_active -> Bool,
        started_at -> Timestamptz,
        ended_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(cities -> provinces (province_id));
diesel::joinable!(organization_positions -> organization_position_levels (organization_position_level_id));
diesel::joinable!(oauth_access_tokens -> oauth_clients (client_id));
diesel::joinable!(oauth_access_tokens -> sys_users (user_id));
diesel::joinable!(oauth_auth_codes -> oauth_clients (client_id));
diesel::joinable!(oauth_auth_codes -> sys_users (user_id));
diesel::joinable!(oauth_clients -> sys_users (user_id));
diesel::joinable!(oauth_personal_access_clients -> oauth_clients (client_id));
diesel::joinable!(oauth_refresh_tokens -> oauth_access_tokens (access_token_id));
diesel::joinable!(provinces -> countries (country_id));
diesel::joinable!(sys_model_has_permissions -> sys_permissions (permission_id));
diesel::joinable!(sys_model_has_roles -> sys_roles (role_id));
diesel::joinable!(user_organizations -> organization_positions (organization_position_id));
diesel::joinable!(user_organizations -> organizations (organization_id));
diesel::joinable!(user_organizations -> sys_users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    cities,
    countries,
    events,
    organization_position_levels,
    organization_positions,
    jobs,
    migrations,
    notifications,
    oauth_access_tokens,
    oauth_auth_codes,
    oauth_clients,
    oauth_personal_access_clients,
    oauth_refresh_tokens,
    oauth_scopes,
    organizations,
    provinces,
    push_subscriptions,
    sys_model_has_permissions,
    sys_model_has_roles,
    sys_permissions,
    sys_roles,
    sys_users,
    user_organizations,
);
