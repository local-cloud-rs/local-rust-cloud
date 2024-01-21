use crate::http::aws::iam::types;
#[derive(Debug, PartialEq, serde::Deserialize)]
pub(crate) struct UntagInstanceProfileRequest {
    #[serde(rename = "InstanceProfileName")]
    pub(crate) instance_profile_name: Option<types::instance_profile_name_type::InstanceProfileNameType>,
    #[serde(rename = "TagKeys")]
    pub(crate) tag_keys: Option<Vec<types::tag_key_type::TagKeyType>>,
}
impl UntagInstanceProfileRequest {
    pub(crate) fn instance_profile_name(&self) -> Option<&str> {
        self.instance_profile_name.as_deref()
    }
    pub(crate) fn tag_keys(&self) -> Option<&[types::tag_key_type::TagKeyType]> {
        self.tag_keys.as_deref()
    }
}
impl local_cloud_validate::NamedValidator for &UntagInstanceProfileRequest {
    fn validate(&self, at: &str) -> Result<(), local_cloud_validate::ValidationError> {
        local_cloud_validate::validate_required(
            self.instance_profile_name(),
            format!("{at}.{}", "InstanceProfileName").as_str(),
        )?;
        local_cloud_validate::validate_named(
            self.instance_profile_name.as_ref(),
            format!("{at}.{}", "InstanceProfileName").as_str(),
        )?;
        local_cloud_validate::validate_required(self.tag_keys(), format!("{at}.{}", "TagKeys").as_str())?;
        local_cloud_validate::validate_array_size_min(self.tag_keys(), 0usize, format!("{at}.{}", "TagKeys").as_str())?;
        local_cloud_validate::validate_array_size_max(
            self.tag_keys(),
            50usize,
            format!("{at}.{}", "TagKeys").as_str(),
        )?;
        if let Some(tag_keys) = self.tag_keys() {
            for (id, member) in tag_keys.iter().enumerate() {
                local_cloud_validate::validate_named(Some(member), format!("{at}.{}.member.{id}", "TagKeys").as_str())?;
            }
        }
        Ok(())
    }
}
