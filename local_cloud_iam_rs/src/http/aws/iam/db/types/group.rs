use aws_sdk_iam::types::Group;
use aws_smithy_types::DateTime;
use derive_builder::Builder;
use sqlx::FromRow;

use crate::http::aws::iam::db::types::common::Pageable;
use crate::http::aws::iam::types::list_groups_request::ListGroupsRequest;

#[derive(Debug, Builder)]
pub(crate) struct InsertGroup {
    pub(crate) id: Option<i64>,
    pub(crate) account_id: i64,
    pub(crate) group_id: String,
    pub(crate) group_name: String,
    pub(crate) arn: String,
    pub(crate) path: String,
    pub(crate) create_date: i64,
}

#[derive(Debug, FromRow)]
pub(crate) struct SelectGroup {
    pub(crate) id: i64,
    pub(crate) account_id: i64,
    pub(crate) group_id: String,
    pub(crate) group_name: String,
    pub(crate) arn: String,
    pub(crate) path: String,
    pub(crate) create_date: i64,
}

#[derive(Debug)]
pub(crate) struct ListGroupsQuery {
    pub(crate) path_prefix: String,
    pub(crate) limit: i32,
    pub(crate) skip: i32,
}

impl Pageable for &ListGroupsQuery {
    fn limit(&self) -> i32 {
        self.limit
    }

    fn skip(&self) -> i32 {
        self.skip
    }
}

impl Into<ListGroupsQuery> for &ListGroupsRequest {
    fn into(self) -> ListGroupsQuery {
        let limit = match self.max_items() {
            None => 10,
            Some(v) => *v,
        };

        let skip = match self.marker_type() {
            None => 0,
            // unwrap is safe since marker must be validated before DB query preparation
            Some(marker_type) => marker_type.marker().unwrap().truncate_amount,
        };

        ListGroupsQuery {
            path_prefix: self.path_prefix().unwrap_or("/").to_owned(),
            limit: if limit < 1 { 10 } else { limit },
            skip,
        }
    }
}

impl Into<Group> for &SelectGroup {
    fn into(self) -> Group {
        Group::builder()
            .group_name(&self.group_name)
            .group_id(&self.group_id)
            .path(&self.path)
            .arn(&self.arn)
            .create_date(DateTime::from_secs(self.create_date))
            .build()
            .unwrap()
    }
}
