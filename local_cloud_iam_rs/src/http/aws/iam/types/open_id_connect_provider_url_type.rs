use std::ops::Deref;

///<p>Contains a URL that specifies the endpoint for an OpenID Connect provider.</p>
#[derive(Debug, PartialEq, serde::Deserialize)]
pub(crate) struct OpenIdConnectProviderUrlType(String);

impl Deref for OpenIdConnectProviderUrlType {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl local_cloud_validate::NamedValidator for &OpenIdConnectProviderUrlType {
    fn validate(&self, at: &str) -> Result<(), local_cloud_validate::ValidationError> {
        local_cloud_validate::validate_str_length_min(Some(&self), 1usize, at)?;
        local_cloud_validate::validate_str_length_max(Some(&self), 255usize, at)?;
        Ok(())
    }
}
