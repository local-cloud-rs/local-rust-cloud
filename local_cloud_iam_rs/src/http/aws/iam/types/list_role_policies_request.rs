use local_cloud_validate::{validate_named, validate_required};

use crate::http::aws::iam::types;
use crate::http::aws::iam::types::marker_type::MarkerType;

#[derive(Debug, PartialEq, serde::Deserialize)]
pub(crate) struct ListRolePoliciesRequest {
    #[serde(rename = "RoleName")]
    pub(crate) role_name: Option<types::role_name_type::RoleNameType>,
    #[serde(rename = "Marker")]
    pub(crate) marker: Option<MarkerType>,
    #[serde(rename = "MaxItems")]
    pub(crate) max_items: Option<types::max_items_type::MaxItemsType>,
}

impl ListRolePoliciesRequest {
    pub(crate) fn role_name(&self) -> Option<&str> {
        self.role_name.as_deref()
    }
    pub(crate) fn marker(&self) -> Option<&str> {
        self.marker.as_deref()
    }
    pub(crate) fn marker_type(&self) -> Option<&MarkerType> {
        self.marker.as_ref()
    }
    pub(crate) fn max_items(&self) -> Option<&i32> {
        self.max_items.as_deref()
    }
}

impl local_cloud_validate::NamedValidator for &ListRolePoliciesRequest {
    fn validate(&self, at: &str) -> Result<(), local_cloud_validate::ValidationError> {
        validate_required(self.role_name(), format!("{at}.{}", "RoleName").as_str())?;
        validate_named(self.role_name.as_ref(), format!("{at}.{}", "RoleName").as_str())?;
        validate_named(self.marker.as_ref(), format!("{at}.{}", "Marker").as_str())?;
        validate_named(self.max_items.as_ref(), format!("{at}.{}", "MaxItems").as_str())?;
        Ok(())
    }
}
