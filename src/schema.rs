// @generated automatically by Diesel CLI.

diesel::table! {
    activity_log (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 255]
        correlation_id -> Nullable<Varchar>,
        #[max_length = 255]
        log_name -> Nullable<Varchar>,
        description -> Text,
        #[max_length = 255]
        subject_type -> Nullable<Varchar>,
        #[max_length = 255]
        subject_id -> Nullable<Varchar>,
        #[max_length = 255]
        causer_type -> Nullable<Varchar>,
        #[max_length = 255]
        causer_id -> Nullable<Varchar>,
        properties -> Nullable<Jsonb>,
        #[max_length = 255]
        batch_uuid -> Nullable<Varchar>,
        #[max_length = 255]
        event -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    algorithm_compatibility_matrix (id) {
        #[max_length = 26]
        id -> Bpchar,
        encryption_algorithm_a -> Varchar,
        encryption_algorithm_b -> Varchar,
        key_exchange_algorithm_a -> Varchar,
        key_exchange_algorithm_b -> Varchar,
        is_compatible -> Bool,
        compatibility_level -> Varchar,
        negotiation_overhead_ms -> Nullable<Int4>,
        interop_test_passed -> Nullable<Bool>,
        tested_at -> Nullable<Timestamptz>,
        test_version -> Varchar,
        notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ref_geo_cities (id) {
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
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    conversation_algorithm_negotiations (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        negotiated_encryption_algorithm -> Varchar,
        negotiated_key_exchange -> Varchar,
        negotiated_mac_algorithm -> Varchar,
        negotiated_protocol_version -> Int4,
        negotiation_started_at -> Timestamptz,
        negotiation_completed_at -> Nullable<Timestamptz>,
        is_negotiation_complete -> Bool,
        negotiation_participants_count -> Int4,
        all_participants_responded -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        negotiated_signature_algorithm -> Nullable<Varchar>,
        negotiated_kdf_algorithm -> Nullable<Varchar>,
        negotiated_pq_kem_algorithm -> Nullable<Varchar>,
        negotiated_pq_signature_algorithm -> Nullable<Varchar>,
    }
}

diesel::table! {
    conversation_device_settings (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        preferred_algorithm -> Nullable<Varchar>,
        preferred_key_exchange -> Nullable<Varchar>,
        preferred_mac -> Nullable<Varchar>,
        supports_disappearing_messages -> Bool,
        supports_file_encryption -> Bool,
        supports_voice_encryption -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    conversation_participants (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        role -> Varchar,
        is_active -> Bool,
        joined_at -> Timestamptz,
        left_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        last_read_message_id -> Nullable<Bpchar>,
        last_read_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    conversations (id) {
        #[max_length = 26]
        id -> Bpchar,
        conversation_type -> Varchar,
        is_encrypted -> Bool,
        encryption_immutable -> Bool,
        encrypted_name -> Nullable<Text>,
        encrypted_description -> Nullable<Text>,
        encrypted_avatar_url -> Nullable<Text>,
        preferred_algorithm -> Nullable<Varchar>,
        preferred_key_exchange -> Nullable<Varchar>,
        preferred_mac -> Nullable<Varchar>,
        #[max_length = 26]
        creator_id -> Nullable<Bpchar>,
        max_participants -> Nullable<Int4>,
        is_public -> Bool,
        disappearing_messages_timer -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    ref_geo_countries (id) {
        #[max_length = 26]
        id -> Bpchar,
        name -> Varchar,
        iso_code -> Varchar,
        phone_code -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    device_algorithm_preferences (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        preferred_encryption_algorithms -> Array<Nullable<Text>>,
        preferred_key_exchange_algorithms -> Array<Nullable<Text>>,
        preferred_mac_algorithms -> Array<Nullable<Text>>,
        allow_algorithm_fallback -> Bool,
        minimum_security_level -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        preferred_signature_algorithms -> Array<Nullable<Text>>,
        preferred_kdf_algorithms -> Array<Nullable<Text>>,
        preferred_pq_kem_algorithms -> Array<Nullable<Text>>,
        preferred_pq_signature_algorithms -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    device_capabilities (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        supports_aes_256_gcm -> Bool,
        supports_chacha20_poly1305 -> Bool,
        supports_aes_128_gcm -> Bool,
        supports_curve25519 -> Bool,
        supports_p256_ecdh -> Bool,
        supports_rsa_2048 -> Bool,
        supports_rsa_4096 -> Bool,
        supports_hmac_sha256 -> Bool,
        supports_hmac_sha384 -> Bool,
        supports_hmac_sha512 -> Bool,
        supports_blake3_mac -> Bool,
        max_signal_protocol_version -> Int4,
        min_signal_protocol_version -> Int4,
        supports_multi_device -> Bool,
        supports_group_messaging -> Bool,
        supports_disappearing_messages -> Bool,
        supports_file_encryption -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        supports_ed25519_signature -> Bool,
        supports_rsa_pss_signature -> Bool,
        supports_ecdsa_p256_signature -> Bool,
        supports_hkdf_sha256 -> Bool,
        supports_hkdf_sha384 -> Bool,
        supports_hkdf_sha512 -> Bool,
        supports_kyber_768 -> Bool,
        supports_dilithium2 -> Bool,
        supports_sphincs_plus -> Bool,
        supports_bike_r4 -> Bool,
    }
}

diesel::table! {
    device_fingerprints (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        identity_key_fingerprint -> Text,
        fingerprint_algorithm -> Varchar,
        is_verified -> Bool,
        #[max_length = 26]
        verified_by_user_id -> Nullable<Bpchar>,
        verified_at -> Nullable<Timestamptz>,
        verification_method -> Nullable<Varchar>,
        trust_score -> Int4,
        trust_last_updated -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    device_key_rotations (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        rotation_type -> Varchar,
        scheduled_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        failure_reason -> Nullable<Text>,
        is_completed -> Bool,
        is_failed -> Bool,
        retry_count -> Int4,
        max_retries -> Int4,
        next_retry_at -> Nullable<Timestamptz>,
        old_key_id -> Nullable<Text>,
        new_key_id -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    device_presence (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        status -> Varchar,
        last_seen_at -> Timestamptz,
        encrypted_status_message -> Nullable<Text>,
        status_message_algorithm -> Nullable<Varchar>,
        auto_away_after_minutes -> Nullable<Int4>,
        auto_offline_after_minutes -> Nullable<Int4>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    device_push_tokens (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        platform -> Varchar,
        token -> Text,
        endpoint -> Nullable<Text>,
        encrypted_notification_settings -> Nullable<Text>,
        settings_algorithm -> Nullable<Varchar>,
        is_active -> Bool,
        last_used_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    device_session_backups (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        backup_name -> Varchar,
        backup_type -> Varchar,
        backup_version -> Int4,
        encrypted_sessions_data -> Text,
        backup_algorithm -> Varchar,
        backup_key_hash -> Text,
        sessions_count -> Int4,
        conversations_count -> Int4,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        last_accessed_at -> Nullable<Timestamptz>,
        backup_checksum -> Text,
        is_verified -> Bool,
        verification_failed_at -> Nullable<Timestamptz>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    device_sync_sessions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        primary_device_id -> Bpchar,
        #[max_length = 26]
        secondary_device_id -> Bpchar,
        encrypted_sync_key -> Text,
        sync_algorithm -> Varchar,
        is_active -> Bool,
        last_sync_at -> Timestamptz,
        established_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    device_verification_codes (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        #[max_length = 26]
        verifying_device_id -> Bpchar,
        #[max_length = 60]
        safety_number -> Varchar,
        verification_method -> Varchar,
        is_verified -> Bool,
        verified_at -> Nullable<Timestamptz>,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    devices (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        device_name -> Varchar,
        device_type -> Varchar,
        identity_public_key -> Text,
        signed_prekey_public -> Text,
        signed_prekey_signature -> Text,
        signed_prekey_id -> Int4,
        supported_algorithms -> Array<Nullable<Text>>,
        is_active -> Bool,
        last_seen_at -> Timestamptz,
        registration_id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        signed_prekey_rotation_needed -> Bool,
        last_key_rotation_at -> Nullable<Timestamptz>,
        prekey_rotation_interval -> Interval,
        trust_level -> Varchar,
    }
}

diesel::table! {
    ref_geo_districts (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        city_id -> Bpchar,
        name -> Varchar,
        code -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    encrypted_backup_keys (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        encrypted_backup_data -> Text,
        backup_algorithm -> Varchar,
        backup_type -> Varchar,
        backup_size_bytes -> Int8,
        #[max_length = 64]
        backup_hash -> Varchar,
        is_verified -> Bool,
        expires_at -> Timestamptz,
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
    forward_history (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        original_message_id -> Bpchar,
        #[max_length = 26]
        forwarded_by_user_id -> Bpchar,
        #[max_length = 26]
        forwarded_by_device_id -> Bpchar,
        forward_depth -> Int4,
        created_at -> Timestamptz,
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
    message_delivery_status (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        recipient_device_id -> Bpchar,
        status -> Varchar,
        delivered_at -> Nullable<Timestamptz>,
        read_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        failure_reason -> Nullable<Varchar>,
        retry_count -> Int4,
        max_retries -> Int4,
        next_retry_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    message_device_keys (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        recipient_device_id -> Bpchar,
        encrypted_message_key -> Text,
        key_algorithm -> Varchar,
        delivered_at -> Nullable<Timestamptz>,
        read_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    message_expiry_queue (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        expires_at -> Timestamptz,
        expiry_type -> Varchar,
        is_processed -> Bool,
        processed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    message_key_gc_policies (id) {
        #[max_length = 26]
        id -> Bpchar,
        policy_name -> Varchar,
        applies_to_table -> Varchar,
        max_age_days -> Int4,
        max_unused_keys -> Nullable<Int4>,
        cleanup_frequency_hours -> Int4,
        cleanup_condition -> Text,
        preserve_condition -> Nullable<Text>,
        is_active -> Bool,
        last_run_at -> Nullable<Timestamptz>,
        next_run_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    message_key_pools (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        session_id -> Bpchar,
        pool_start_index -> Int4,
        pool_end_index -> Int4,
        pool_size -> Int4,
        encrypted_key_pool -> Text,
        pool_algorithm -> Varchar,
        #[max_length = 26]
        sender_device_id -> Bpchar,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        keys_used_count -> Int4,
        is_pool_exhausted -> Bool,
        gc_eligible_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    message_mentions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        mentioned_user_id -> Bpchar,
        mention_type -> Varchar,
        mention_start_pos -> Nullable<Int4>,
        mention_length -> Nullable<Int4>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    message_reactions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        encrypted_reaction -> Text,
        reaction_algorithm -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    messages (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        #[max_length = 26]
        sender_user_id -> Bpchar,
        #[max_length = 26]
        sender_device_id -> Bpchar,
        message_type -> Varchar,
        encrypted_content -> Text,
        content_algorithm -> Varchar,
        #[max_length = 26]
        reply_to_message_id -> Nullable<Bpchar>,
        #[max_length = 26]
        forward_from_message_id -> Nullable<Bpchar>,
        #[max_length = 26]
        edit_of_message_id -> Nullable<Bpchar>,
        is_edited -> Bool,
        is_deleted -> Bool,
        expires_at -> Nullable<Timestamptz>,
        sent_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    mfa_attempts (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 50]
        method_type -> Varchar,
        ip_address -> Nullable<Text>,
        user_agent -> Nullable<Text>,
        success -> Bool,
        attempted_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    mfa_methods (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 50]
        method_type -> Varchar,
        secret -> Nullable<Text>,
        is_enabled -> Bool,
        is_verified -> Bool,
        backup_codes -> Nullable<Jsonb>,
        recovery_codes_used_count -> Int4,
        last_used_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
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
        #[max_length = 255]
        jwk_thumbprint -> Nullable<Varchar>,
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
    oauth_ciba_auth_codes (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        ciba_request_id -> Bpchar,
        #[max_length = 255]
        code -> Varchar,
        #[max_length = 26]
        client_id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        scopes -> Nullable<Text>,
        #[max_length = 2048]
        redirect_uri -> Nullable<Varchar>,
        #[max_length = 255]
        code_challenge -> Nullable<Varchar>,
        #[max_length = 10]
        code_challenge_method -> Nullable<Varchar>,
        expires_at -> Timestamptz,
        revoked -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_ciba_requests (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 255]
        auth_req_id -> Varchar,
        #[max_length = 26]
        client_id -> Bpchar,
        #[max_length = 26]
        user_id -> Nullable<Bpchar>,
        #[max_length = 255]
        scope -> Nullable<Varchar>,
        #[max_length = 255]
        binding_message -> Nullable<Varchar>,
        #[max_length = 255]
        user_code -> Nullable<Varchar>,
        #[max_length = 255]
        login_hint -> Nullable<Varchar>,
        login_hint_token -> Nullable<Text>,
        id_token_hint -> Nullable<Text>,
        requested_expiry -> Nullable<Int4>,
        #[max_length = 50]
        status -> Varchar,
        #[max_length = 255]
        notification_endpoint -> Nullable<Varchar>,
        notification_token -> Nullable<Text>,
        expires_at -> Timestamptz,
        interval_seconds -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        authorized_at -> Nullable<Timestamptz>,
        denied_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    oauth_client_certificates (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        client_id -> Bpchar,
        #[max_length = 500]
        subject_dn -> Varchar,
        #[max_length = 64]
        thumbprint_sha256 -> Varchar,
        #[max_length = 500]
        issuer_dn -> Varchar,
        #[max_length = 100]
        serial_number -> Varchar,
        valid_from -> Timestamptz,
        valid_to -> Timestamptz,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    oauth_clients (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        organization_id -> Nullable<Bpchar>,
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
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
        public_key_pem -> Nullable<Text>,
        metadata -> Jsonb,
        #[max_length = 2048]
        jwks_uri -> Nullable<Varchar>,
        #[max_length = 50]
        token_endpoint_auth_method -> Varchar,
        response_types -> Array<Nullable<Text>>,
        grant_types -> Array<Nullable<Text>>,
        #[max_length = 1000]
        scope -> Varchar,
        audience -> Nullable<Array<Nullable<Text>>>,
        require_auth_time -> Bool,
        default_max_age -> Nullable<Int4>,
        require_pushed_authorization_requests -> Bool,
        certificate_bound_access_tokens -> Bool,
    }
}

diesel::table! {
    oauth_device_codes (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 64]
        device_code -> Varchar,
        #[max_length = 9]
        user_code -> Varchar,
        #[max_length = 26]
        client_id -> Bpchar,
        #[max_length = 26]
        user_id -> Nullable<Bpchar>,
        scopes -> Nullable<Text>,
        #[max_length = 255]
        verification_uri -> Varchar,
        #[max_length = 512]
        verification_uri_complete -> Nullable<Varchar>,
        expires_at -> Timestamptz,
        interval -> Int4,
        user_authorized -> Bool,
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
    oauth_pushed_requests (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 255]
        request_uri -> Varchar,
        #[max_length = 26]
        client_id -> Bpchar,
        request_data -> Text,
        expires_at -> Timestamptz,
        used -> Bool,
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
    organization_position_levels (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        organization_id -> Bpchar,
        code -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        level -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    organization_positions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        organization_id -> Bpchar,
        #[max_length = 26]
        organization_position_level_id -> Bpchar,
        code -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        is_active -> Bool,
        min_salary -> Numeric,
        max_salary -> Numeric,
        max_incumbents -> Int4,
        qualifications -> Jsonb,
        responsibilities -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    organizations (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        parent_id -> Nullable<Bpchar>,
        code -> Nullable<Varchar>,
        name -> Varchar,
        address -> Nullable<Text>,
        authorized_capital -> Nullable<Numeric>,
        business_activities -> Nullable<Text>,
        contact_persons -> Nullable<Jsonb>,
        description -> Nullable<Text>,
        email -> Nullable<Varchar>,
        establishment_date -> Nullable<Date>,
        governance_structure -> Nullable<Jsonb>,
        legal_status -> Nullable<Varchar>,
        paid_capital -> Nullable<Numeric>,
        path -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        registration_number -> Nullable<Varchar>,
        tax_number -> Nullable<Varchar>,
        website -> Nullable<Varchar>,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    pinned_messages (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        pinned_by_user_id -> Bpchar,
        #[max_length = 26]
        pinned_by_device_id -> Bpchar,
        pinned_at -> Timestamptz,
        unpinned_at -> Nullable<Timestamptz>,
        is_active -> Bool,
    }
}

diesel::table! {
    poll_votes (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        poll_id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        encrypted_vote_data -> Text,
        vote_algorithm -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    polls (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        encrypted_question -> Text,
        encrypted_options -> Text,
        allows_multiple_votes -> Bool,
        is_anonymous -> Bool,
        expires_at -> Nullable<Timestamptz>,
        is_closed -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    prekey_bundles (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        prekey_id -> Int4,
        prekey_public -> Text,
        is_used -> Bool,
        used_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        used_by_user_id -> Nullable<Bpchar>,
        #[max_length = 26]
        used_by_device_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    ref_geo_provinces (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        country_id -> Bpchar,
        name -> Varchar,
        code -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
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
        failed_attempts -> Int4,
        last_error -> Nullable<Text>,
        last_failed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    scheduled_message_edits (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        scheduled_message_id -> Bpchar,
        new_encrypted_content -> Text,
        new_content_algorithm -> Varchar,
        #[max_length = 26]
        edited_by_device_id -> Bpchar,
        is_applied -> Bool,
        applied_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    scheduled_messages (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        message_id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        #[max_length = 26]
        sender_user_id -> Bpchar,
        #[max_length = 26]
        sender_device_id -> Bpchar,
        scheduled_for -> Timestamptz,
        #[max_length = 50]
        timezone -> Varchar,
        is_sent -> Bool,
        sent_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        failure_reason -> Nullable<Text>,
        retry_count -> Int4,
        max_retries -> Int4,
        next_retry_at -> Nullable<Timestamptz>,
        is_cancelled -> Bool,
        cancelled_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        cancelled_by_device_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    security_incidents (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        device_id -> Nullable<Bpchar>,
        #[max_length = 26]
        user_id -> Nullable<Bpchar>,
        #[max_length = 26]
        conversation_id -> Nullable<Bpchar>,
        incident_type -> Varchar,
        severity -> Varchar,
        encrypted_incident_data -> Nullable<Text>,
        incident_algorithm -> Nullable<Varchar>,
        is_resolved -> Bool,
        resolved_at -> Nullable<Timestamptz>,
        resolution_notes -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sender_key_sessions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        #[max_length = 26]
        sender_device_id -> Bpchar,
        encrypted_sender_key_state -> Text,
        key_algorithm -> Varchar,
        key_generation -> Int4,
        is_active -> Bool,
        created_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    session_recovery_log (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        session_id -> Bpchar,
        #[max_length = 26]
        requesting_device_id -> Bpchar,
        recovery_method -> Varchar,
        recovery_status -> Varchar,
        initiated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        failed_at -> Nullable<Timestamptz>,
        failure_reason -> Nullable<Text>,
        recovery_key_verified -> Bool,
        device_authorized -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    sessions (id) {
        #[max_length = 40]
        id -> Varchar,
        #[max_length = 26]
        user_id -> Nullable<Bpchar>,
        #[max_length = 45]
        ip_address -> Nullable<Varchar>,
        user_agent -> Nullable<Text>,
        payload -> Text,
        last_activity -> Int4,
    }
}

diesel::table! {
    signal_sessions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        local_device_id -> Bpchar,
        #[max_length = 26]
        remote_device_id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        encrypted_session_state -> Text,
        session_algorithm -> Varchar,
        session_version -> Int4,
        is_active -> Bool,
        established_at -> Timestamptz,
        last_used_at -> Timestamptz,
        encrypted_send_counter -> Nullable<Text>,
        encrypted_receive_counter -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        backup_encrypted_state -> Nullable<Text>,
        recovery_key_hash -> Nullable<Text>,
        backup_created_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        backup_device_id -> Nullable<Bpchar>,
        is_recoverable -> Bool,
    }
}

diesel::table! {
    skipped_message_keys (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        session_id -> Bpchar,
        encrypted_message_key -> Text,
        key_algorithm -> Varchar,
        message_number -> Int4,
        chain_key_index -> Int4,
        header_key -> Text,
        #[max_length = 26]
        sender_device_id -> Bpchar,
        #[max_length = 26]
        expected_message_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
        used_at -> Nullable<Timestamptz>,
        is_used -> Bool,
        gc_eligible_at -> Nullable<Timestamptz>,
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
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Bpchar,
        #[max_length = 26]
        updated_by_id -> Bpchar,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
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
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Bpchar,
        #[max_length = 26]
        updated_by_id -> Bpchar,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    sys_permissions (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        organization_id -> Nullable<Bpchar>,
        guard_name -> Varchar,
        resource -> Nullable<Varchar>,
        action -> Varchar,
        #[max_length = 255]
        scope_type -> Nullable<Varchar>,
        #[max_length = 26]
        scope_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 26]
        created_by_id -> Bpchar,
        #[max_length = 26]
        updated_by_id -> Bpchar,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    sys_roles (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        organization_id -> Nullable<Bpchar>,
        name -> Varchar,
        description -> Nullable<Text>,
        guard_name -> Varchar,
        #[max_length = 255]
        scope_type -> Nullable<Varchar>,
        #[max_length = 26]
        scope_id -> Nullable<Bpchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Bpchar,
        #[max_length = 26]
        updated_by_id -> Bpchar,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
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
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
        email_notifications -> Nullable<Bool>,
        database_notifications -> Nullable<Bool>,
        broadcast_notifications -> Nullable<Bool>,
        web_push_notifications -> Nullable<Bool>,
        sms_notifications -> Nullable<Bool>,
        slack_notifications -> Nullable<Bool>,
        marketing_emails -> Nullable<Bool>,
        security_alerts -> Nullable<Bool>,
        order_updates -> Nullable<Bool>,
        newsletter -> Nullable<Bool>,
        promotional_emails -> Nullable<Bool>,
        account_notifications -> Nullable<Bool>,
        identity_public_key -> Nullable<Text>,
        identity_key_created_at -> Nullable<Timestamptz>,
        mfa_enabled -> Bool,
        mfa_secret -> Nullable<Text>,
        mfa_backup_codes -> Nullable<Jsonb>,
        mfa_required -> Bool,
        push_notifications -> Bool,
        system_updates -> Bool,
        mention_notifications -> Bool,
        comment_notifications -> Bool,
        follow_notifications -> Bool,
    }
}

diesel::table! {
    typing_indicators (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        conversation_id -> Bpchar,
        #[max_length = 26]
        user_id -> Bpchar,
        #[max_length = 26]
        device_id -> Bpchar,
        is_typing -> Bool,
        started_typing_at -> Timestamptz,
        expires_at -> Timestamptz,
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
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::table! {
    ref_geo_villages (id) {
        #[max_length = 26]
        id -> Bpchar,
        #[max_length = 26]
        district_id -> Bpchar,
        name -> Varchar,
        code -> Nullable<Varchar>,
        latitude -> Nullable<Numeric>,
        longitude -> Nullable<Numeric>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
        #[max_length = 26]
        created_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        updated_by_id -> Nullable<Bpchar>,
        #[max_length = 26]
        deleted_by_id -> Nullable<Bpchar>,
    }
}

diesel::joinable!(ref_geo_cities -> ref_geo_provinces (province_id));
diesel::joinable!(conversation_algorithm_negotiations -> conversations (conversation_id));
diesel::joinable!(conversation_device_settings -> conversations (conversation_id));
diesel::joinable!(conversation_device_settings -> devices (device_id));
diesel::joinable!(conversation_participants -> conversations (conversation_id));
diesel::joinable!(conversation_participants -> messages (last_read_message_id));
diesel::joinable!(conversation_participants -> sys_users (user_id));
diesel::joinable!(conversations -> sys_users (creator_id));
diesel::joinable!(device_algorithm_preferences -> devices (device_id));
diesel::joinable!(device_capabilities -> devices (device_id));
diesel::joinable!(device_fingerprints -> devices (device_id));
diesel::joinable!(device_fingerprints -> sys_users (verified_by_user_id));
diesel::joinable!(device_key_rotations -> devices (device_id));
diesel::joinable!(device_presence -> devices (device_id));
diesel::joinable!(device_push_tokens -> devices (device_id));
diesel::joinable!(device_session_backups -> devices (device_id));
diesel::joinable!(device_session_backups -> sys_users (user_id));
diesel::joinable!(devices -> sys_users (user_id));
diesel::joinable!(ref_geo_districts -> ref_geo_cities (city_id));
diesel::joinable!(encrypted_backup_keys -> devices (device_id));
diesel::joinable!(encrypted_backup_keys -> sys_users (user_id));
diesel::joinable!(forward_history -> devices (forwarded_by_device_id));
diesel::joinable!(forward_history -> sys_users (forwarded_by_user_id));
diesel::joinable!(message_delivery_status -> devices (recipient_device_id));
diesel::joinable!(message_delivery_status -> messages (message_id));
diesel::joinable!(message_device_keys -> devices (recipient_device_id));
diesel::joinable!(message_device_keys -> messages (message_id));
diesel::joinable!(message_expiry_queue -> messages (message_id));
diesel::joinable!(message_key_pools -> devices (sender_device_id));
diesel::joinable!(message_key_pools -> signal_sessions (session_id));
diesel::joinable!(message_mentions -> messages (message_id));
diesel::joinable!(message_mentions -> sys_users (mentioned_user_id));
diesel::joinable!(message_reactions -> devices (device_id));
diesel::joinable!(message_reactions -> messages (message_id));
diesel::joinable!(message_reactions -> sys_users (user_id));
diesel::joinable!(messages -> conversations (conversation_id));
diesel::joinable!(messages -> devices (sender_device_id));
diesel::joinable!(messages -> sys_users (sender_user_id));
diesel::joinable!(mfa_attempts -> sys_users (user_id));
diesel::joinable!(mfa_methods -> sys_users (user_id));
diesel::joinable!(oauth_access_tokens -> oauth_clients (client_id));
diesel::joinable!(oauth_access_tokens -> sys_users (user_id));
diesel::joinable!(oauth_auth_codes -> oauth_clients (client_id));
diesel::joinable!(oauth_auth_codes -> sys_users (user_id));
diesel::joinable!(oauth_ciba_auth_codes -> oauth_ciba_requests (ciba_request_id));
diesel::joinable!(oauth_ciba_auth_codes -> oauth_clients (client_id));
diesel::joinable!(oauth_ciba_auth_codes -> sys_users (user_id));
diesel::joinable!(oauth_ciba_requests -> oauth_clients (client_id));
diesel::joinable!(oauth_ciba_requests -> sys_users (user_id));
diesel::joinable!(oauth_client_certificates -> oauth_clients (client_id));
diesel::joinable!(oauth_clients -> organizations (organization_id));
diesel::joinable!(oauth_device_codes -> oauth_clients (client_id));
diesel::joinable!(oauth_device_codes -> sys_users (user_id));
diesel::joinable!(oauth_personal_access_clients -> oauth_clients (client_id));
diesel::joinable!(oauth_pushed_requests -> oauth_clients (client_id));
diesel::joinable!(oauth_refresh_tokens -> oauth_access_tokens (access_token_id));
diesel::joinable!(organization_position_levels -> organizations (organization_id));
diesel::joinable!(organization_positions -> organization_position_levels (organization_position_level_id));
diesel::joinable!(organization_positions -> organizations (organization_id));
diesel::joinable!(pinned_messages -> conversations (conversation_id));
diesel::joinable!(pinned_messages -> devices (pinned_by_device_id));
diesel::joinable!(pinned_messages -> messages (message_id));
diesel::joinable!(pinned_messages -> sys_users (pinned_by_user_id));
diesel::joinable!(poll_votes -> devices (device_id));
diesel::joinable!(poll_votes -> polls (poll_id));
diesel::joinable!(poll_votes -> sys_users (user_id));
diesel::joinable!(polls -> conversations (conversation_id));
diesel::joinable!(polls -> messages (message_id));
diesel::joinable!(ref_geo_provinces -> ref_geo_countries (country_id));
diesel::joinable!(scheduled_message_edits -> devices (edited_by_device_id));
diesel::joinable!(scheduled_message_edits -> scheduled_messages (scheduled_message_id));
diesel::joinable!(scheduled_messages -> conversations (conversation_id));
diesel::joinable!(scheduled_messages -> messages (message_id));
diesel::joinable!(scheduled_messages -> sys_users (sender_user_id));
diesel::joinable!(security_incidents -> conversations (conversation_id));
diesel::joinable!(security_incidents -> devices (device_id));
diesel::joinable!(security_incidents -> sys_users (user_id));
diesel::joinable!(sender_key_sessions -> conversations (conversation_id));
diesel::joinable!(sender_key_sessions -> devices (sender_device_id));
diesel::joinable!(session_recovery_log -> devices (requesting_device_id));
diesel::joinable!(session_recovery_log -> signal_sessions (session_id));
diesel::joinable!(sessions -> sys_users (user_id));
diesel::joinable!(signal_sessions -> conversations (conversation_id));
diesel::joinable!(skipped_message_keys -> devices (sender_device_id));
diesel::joinable!(skipped_message_keys -> signal_sessions (session_id));
diesel::joinable!(sys_model_has_permissions -> sys_permissions (permission_id));
diesel::joinable!(sys_model_has_roles -> sys_roles (role_id));
diesel::joinable!(sys_permissions -> organizations (organization_id));
diesel::joinable!(sys_roles -> organizations (organization_id));
diesel::joinable!(typing_indicators -> conversations (conversation_id));
diesel::joinable!(typing_indicators -> devices (device_id));
diesel::joinable!(typing_indicators -> sys_users (user_id));
diesel::joinable!(user_organizations -> organization_positions (organization_position_id));
diesel::joinable!(user_organizations -> organizations (organization_id));
diesel::joinable!(ref_geo_villages -> ref_geo_districts (district_id));

diesel::allow_tables_to_appear_in_same_query!(
    activity_log,
    algorithm_compatibility_matrix,
    ref_geo_cities,
    conversation_algorithm_negotiations,
    conversation_device_settings,
    conversation_participants,
    conversations,
    ref_geo_countries,
    device_algorithm_preferences,
    device_capabilities,
    device_fingerprints,
    device_key_rotations,
    device_presence,
    device_push_tokens,
    device_session_backups,
    device_sync_sessions,
    device_verification_codes,
    devices,
    ref_geo_districts,
    encrypted_backup_keys,
    events,
    forward_history,
    jobs,
    message_delivery_status,
    message_device_keys,
    message_expiry_queue,
    message_key_gc_policies,
    message_key_pools,
    message_mentions,
    message_reactions,
    messages,
    mfa_attempts,
    mfa_methods,
    migrations,
    notifications,
    oauth_access_tokens,
    oauth_auth_codes,
    oauth_ciba_auth_codes,
    oauth_ciba_requests,
    oauth_client_certificates,
    oauth_clients,
    oauth_device_codes,
    oauth_personal_access_clients,
    oauth_pushed_requests,
    oauth_refresh_tokens,
    oauth_scopes,
    organization_position_levels,
    organization_positions,
    organizations,
    pinned_messages,
    poll_votes,
    polls,
    prekey_bundles,
    ref_geo_provinces,
    push_subscriptions,
    scheduled_message_edits,
    scheduled_messages,
    security_incidents,
    sender_key_sessions,
    session_recovery_log,
    sessions,
    signal_sessions,
    skipped_message_keys,
    sys_model_has_permissions,
    sys_model_has_roles,
    sys_permissions,
    sys_roles,
    sys_users,
    typing_indicators,
    user_organizations,
    ref_geo_villages,
);
