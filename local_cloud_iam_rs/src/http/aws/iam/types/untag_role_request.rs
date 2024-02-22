use local_cloud_validate::{validate_array_size_max, validate_array_size_min, validate_named, validate_required};

use crate::http::aws::iam::types;
use crate::http::aws::iam::types::tag_key_type::TagKeyType;

#[derive(Debug, PartialEq, serde::Deserialize)]
pub(crate) struct UntagRoleRequest {
    #[serde(rename = "TagKeys")]
    pub(crate) tag_keys: Option<Vec<TagKeyType>>,
    #[serde(rename = "RoleName")]
    pub(crate) role_name: Option<types::role_name_type::RoleNameType>,
}

impl UntagRoleRequest {
    pub(crate) fn tag_keys(&self) -> Vec<String> {
        match &self.tag_keys {
            None => vec![],
            Some(keys) => keys.iter().map(|k| k.to_string()).collect::<Vec<_>>(),
        }
    }
    pub(crate) fn tag_keys_type(&self) -> Option<&[TagKeyType]> {
        self.tag_keys.as_deref()
    }
    pub(crate) fn role_name(&self) -> Option<&str> {
        self.role_name.as_deref()
    }
}

impl local_cloud_validate::NamedValidator for &UntagRoleRequest {
    fn validate(&self, at: &str) -> Result<(), local_cloud_validate::ValidationError> {
        validate_required(self.tag_keys_type(), format!("{at}.{}", "TagKeys").as_str())?;
        validate_array_size_min(self.tag_keys_type(), 0usize, format!("{at}.{}", "TagKeys").as_str())?;
        validate_array_size_max(self.tag_keys_type(), 50usize, format!("{at}.{}", "TagKeys").as_str())?;
        if let Some(tag_keys) = self.tag_keys_type() {
            for (id, member) in tag_keys.iter().enumerate() {
                validate_named(Some(member), format!("{at}.{}.member.{id}", "TagKeys").as_str())?;
            }
        }
        validate_required(self.role_name(), format!("{at}.{}", "RoleName").as_str())?;
        validate_named(self.role_name.as_ref(), format!("{at}.{}", "RoleName").as_str())?;
        Ok(())
    }
}
