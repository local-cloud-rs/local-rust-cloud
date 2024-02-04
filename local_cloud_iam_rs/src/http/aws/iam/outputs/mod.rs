mod access_details;
mod access_key_metadata;
mod access_keys;
mod add_role_to_instance_profile;
mod add_user_to_group;
mod attach_group_policy;
mod attach_role_policy;
mod attach_user_policy;
mod attached_permissions_boundaries;
mod attached_policies;
mod change_password;
mod confirmation;
mod create_access_key;
mod create_account_alias;
mod create_group;
mod create_instance_profile;
mod create_login_profile;
mod create_open_id_connect_provider;
mod create_policy;
mod create_policy_version;
mod create_role;
mod create_saml_provider;
mod create_service_linked_role;
mod create_service_specific_credential;
mod create_user;
mod create_virtual_mfa_device;
mod deactivate_mfa_device;
mod delete_access_key;
mod delete_account_alias;
mod delete_account_password_policy;
mod delete_group;
mod delete_group_policy;
mod delete_instance_profile;
mod delete_login_profile;
mod delete_open_id_connect_provider;
mod delete_policy;
mod delete_policy_version;
mod delete_role;
mod delete_role_permissions_boundary;
mod delete_role_policy;
mod delete_saml_provider;
mod delete_server_certificate;
mod delete_service_linked_role;
mod delete_service_specific_credential;
mod delete_signing_certificate;
mod delete_ssh_public_key;
mod delete_user;
mod delete_user_permissions_boundary;
mod delete_user_policy;
mod delete_virtual_mfa_device;
mod detach_group_policy;
mod detach_role_policy;
mod detach_user_policy;
mod enable_mfa_device;
mod entity_details;
mod entity_info;
mod error_details;
mod evaluation_results;
mod generate_credential_report;
mod generate_organizations_access_report;
mod generate_service_last_accessed_details;
mod get_access_key_last_used;
mod get_account_authorization_details;
mod get_account_password_policy;
mod get_account_summary;
mod get_context_keys_for_custom_policy;
mod get_context_keys_for_principal_policy;
mod get_credential_report;
mod get_group;
mod get_group_policy;
mod get_instance_profile;
mod get_login_profile;
mod get_mfa_device;
mod get_open_id_connect_provider;
mod get_organizations_access_report;
mod get_policy;
mod get_policy_version;
mod get_role;
mod get_role_policy;
mod get_saml_provider;
mod get_server_certificate;
mod get_service_last_accessed_details;
mod get_service_last_accessed_details_with_entities;
mod get_service_linked_role_deletion_status;
mod get_ssh_public_key;
mod get_user;
mod get_user_policy;
mod group_details;
mod groups;
mod instance_profiles;
mod list_access_keys;
mod list_account_aliases;
mod list_attached_group_policies;
mod list_attached_role_policies;
mod list_attached_user_policies;
mod list_entities_for_policy;
mod list_group_policies;
mod list_groups;
mod list_groups_for_user;
mod list_instance_profile_tags;
mod list_instance_profiles;
mod list_instance_profiles_for_role;
mod list_mfa_device_tags;
mod list_mfa_devices;
mod list_open_id_connect_provider_tags;
mod list_open_id_connect_providers;
mod list_policies;
mod list_policies_granting_service_access;
mod list_policies_granting_service_access_entries;
mod list_policy_tags;
mod list_policy_versions;
mod list_role_policies;
mod list_role_tags;
mod list_roles;
mod list_saml_provider_tags;
mod list_saml_providers;
mod list_server_certificate_tags;
mod list_server_certificates;
mod list_signing_certificates;
mod list_ssh_public_keys;
mod list_user_policies;
mod list_user_tags;
mod list_users;
mod list_virtual_mfa_devices;
mod managed_policy_details;
mod mfa_devices;
mod open_id_connect_providers;
mod organizations_decision_details;
mod permissions_boundary_decision_details;
mod policies;
mod policy_details;
mod policy_evalution_decision_map;
mod policy_granting_service_access;
mod policy_groups;
mod policy_roles;
mod policy_users;
mod policy_versions;
mod positions;
mod put_group_policy;
mod put_role_permissions_boundary;
mod put_role_policy;
mod put_user_permissions_boundary;
mod put_user_policy;
mod remove_client_id_from_open_id_connect_provider;
mod remove_role_from_instance_profile;
mod remove_user_from_group;
mod reset_service_specific_credential;
mod resource_specific_results;
mod resync_mfa_device;
mod role_details;
mod role_usages;
mod roles;
mod saml_provider_list;
mod server_certificate_metadata;
mod server_certificates;
mod service_specific_credentials;
mod services_last_accessed;
mod set_default_policy_version;
mod set_security_token_service_preferences;
mod signing_certificates;
mod simulate_custom_policy;
mod simulate_principal_policy;
mod ssh_public_key_metadata;
mod ssh_public_keys;
mod statements;
mod strings;
mod tag_instance_profile;
mod tag_mfa_device;
mod tag_open_id_connect_provider;
mod tag_policy;
mod tag_role;
mod tag_saml_provider;
mod tag_server_certificate;
mod tag_user;
mod tags;
mod tracked_actions_last_accessed;
mod untag_instance_profile;
mod untag_mfa_device;
mod untag_open_id_connect_provider;
mod untag_policy;
mod untag_role;
mod untag_saml_provider;
mod untag_server_certificate;
mod untag_user;
mod update_access_key;
mod update_account_password_policy;
mod update_assume_role_policy;
mod update_group;
mod update_login_profile;
mod update_open_id_connect_provider_thumbprint;
mod update_role;
mod update_role_description;
mod update_saml_provider;
mod update_server_certificate;
mod update_service_specific_credential;
mod update_signing_certificate;
mod update_ssh_public_key;
mod update_user;
mod upload_server_certificate;
mod upload_signing_certificate;
mod upload_ssh_public_key;
mod user_details;
mod users;
mod virtual_mfa_devices;
pub(crate) mod wrapper;
