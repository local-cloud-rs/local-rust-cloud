use aws_sdk_iam::operation::attach_role_policy::AttachRolePolicyOutput;
use aws_sdk_iam::operation::create_role::CreateRoleOutput;
use aws_sdk_iam::operation::get_role_policy::GetRolePolicyOutput;
use aws_sdk_iam::operation::list_role_policies::ListRolePoliciesOutput;
use aws_sdk_iam::operation::list_role_tags::ListRoleTagsOutput;
use aws_sdk_iam::operation::list_roles::ListRolesOutput;
use aws_sdk_iam::operation::put_role_policy::PutRolePolicyOutput;
use aws_sdk_iam::operation::tag_role::TagRoleOutput;
use aws_sdk_iam::operation::untag_role::UntagRoleOutput;
use aws_sdk_iam::types::Role;
use aws_smithy_types::DateTime;
use chrono::Utc;
use sqlx::{Executor, Sqlite};

use local_cloud_db::LocalDb;
use local_cloud_validate::NamedValidator;

use crate::http::aws::iam::actions::error::ApiErrorKind;
use crate::http::aws::iam::db::types::inline_policy::{DbInlinePolicy, ListInlinePoliciesQuery};
use crate::http::aws::iam::db::types::resource_identifier::ResourceType;
use crate::http::aws::iam::db::types::role::{InsertRole, InsertRoleBuilder, InsertRoleBuilderError, SelectRole};
use crate::http::aws::iam::db::types::tags::ListTagsQuery;
use crate::http::aws::iam::operations::common::create_resource_id;
use crate::http::aws::iam::operations::ctx::OperationCtx;
use crate::http::aws::iam::operations::error::OperationError;
use crate::http::aws::iam::types::attach_role_policy_request::AttachRolePolicyRequest;
use crate::http::aws::iam::types::create_role_request::CreateRoleRequest;
use crate::http::aws::iam::types::get_role_policy_request::GetRolePolicyRequest;
use crate::http::aws::iam::types::list_role_policies_request::ListRolePoliciesRequest;
use crate::http::aws::iam::types::list_role_tags_request::ListRoleTagsRequest;
use crate::http::aws::iam::types::list_roles_request::ListRolesRequest;
use crate::http::aws::iam::types::put_role_policy_request::PutRolePolicyRequest;
use crate::http::aws::iam::types::tag_role_request::TagRoleRequest;
use crate::http::aws::iam::types::untag_role_request::UntagRoleRequest;
use crate::http::aws::iam::{constants, db};

pub async fn create_role(
    ctx: &OperationCtx, input: &CreateRoleRequest, db: &LocalDb,
) -> Result<CreateRoleOutput, OperationError> {
    input.validate("$")?;

    let current_time = Utc::now().timestamp();

    let mut tx = db.new_tx().await?;
    let role_id = create_resource_id(&mut tx, constants::role::PREFIX, ResourceType::Role).await?;

    let policy_id = match input.permissions_boundary() {
        None => None,
        Some(permissions_boundary) => {
            Some(super::policy::find_id_by_arn(tx.as_mut(), ctx.account_id, permissions_boundary).await?)
        }
    };

    let mut insert_role = prepare_role_for_insert(ctx, input, &role_id, policy_id, current_time)
        .map_err(|err| OperationError::new(ApiErrorKind::ServiceFailure, err.to_string().as_str()))?;

    db::role::create(&mut tx, &mut insert_role).await?;

    let mut role_tags = super::tag::prepare_for_db(input.tags(), insert_role.id.unwrap());
    db::Tags::Role.save_all(&mut tx, &mut role_tags).await?;

    let role = Role::builder()
        .role_id(role_id)
        .set_assume_role_policy_document(
            input
                .assume_role_policy_document()
                .map(|s| urlencoding::encode(s).to_string()),
        )
        .role_name(&insert_role.role_name)
        .path(&insert_role.path)
        .arn(&insert_role.arn)
        .set_description(insert_role.description.as_ref().map(|s| s.to_owned()))
        .create_date(DateTime::from_secs(insert_role.create_date))
        .set_tags(super::tag::prepare_for_output(&role_tags))
        .build()
        .unwrap();
    let output = CreateRoleOutput::builder().role(role).build();

    tx.commit().await?;
    Ok(output)
}

fn prepare_role_for_insert(
    ctx: &OperationCtx, input: &CreateRoleRequest, role_id: &str, policy_id: Option<i64>, current_time: i64,
) -> Result<InsertRole, InsertRoleBuilderError> {
    let path = input.path().unwrap_or("/");
    let role_name = input.role_name().unwrap().trim();
    let arn = format!("arn:aws:iam::{:0>12}:role{}{}", ctx.account_id, path, role_name);
    let max_session_duration = input
        .max_session_duration()
        .map(|v| *v)
        .unwrap_or(constants::role::DEFAULT_MAX_SESSION_DURATION) as i64;
    InsertRoleBuilder::default()
        .id(None)
        .account_id(ctx.account_id)
        .role_name(role_name.to_owned())
        .assume_role_policy_document(input.assume_role_policy_document().unwrap().to_owned())
        .description(input.description().map(|s| s.to_owned()))
        .max_session_duration(max_session_duration)
        .arn(arn)
        .path(path.to_owned())
        .role_id(role_id.to_owned())
        .policy_id(policy_id)
        .create_date(current_time)
        .build()
}

pub(crate) async fn find_id_by_name<'a, E>(executor: E, account_id: i64, role_name: &str) -> Result<i64, OperationError>
where
    E: 'a + Executor<'a, Database = Sqlite>,
{
    match db::role::find_id_by_name(executor, account_id, role_name).await? {
        Some(role_id) => Ok(role_id),
        None => {
            return Err(OperationError::new(
                ApiErrorKind::NoSuchEntity,
                format!("IAM role with name '{}' doesn't exist.", role_name).as_str(),
            ));
        }
    }
}

pub(crate) async fn attach_role_policy(
    ctx: &OperationCtx, input: &AttachRolePolicyRequest, db: &LocalDb,
) -> Result<AttachRolePolicyOutput, OperationError> {
    input.validate("$")?;

    let mut tx = db.new_tx().await?;

    let found_role_id = find_id_by_name(tx.as_mut(), ctx.account_id, input.role_name().unwrap().trim()).await?;
    let policy_arn = input.policy_arn().unwrap();
    let found_policy_id = super::policy::find_id_by_arn(tx.as_mut(), ctx.account_id, policy_arn).await?;

    db::role::assign_policy_to_role(&mut tx, found_role_id, found_policy_id).await?;

    let output = AttachRolePolicyOutput::builder().build();

    tx.commit().await?;
    Ok(output)
}

pub(crate) async fn list_role_tags(
    ctx: &OperationCtx, input: &ListRoleTagsRequest, db: &LocalDb,
) -> Result<ListRoleTagsOutput, OperationError> {
    input.validate("$")?;

    // obtain connection
    let mut connection = db.new_connection().await?;

    let found_role_id = find_id_by_name(connection.as_mut(), ctx.account_id, input.role_name().unwrap().trim()).await?;

    let query = ListTagsQuery::new(input.max_items(), input.marker_type());
    let found_tags = db::Tags::Role.list(connection.as_mut(), found_role_id, &query).await?;

    let tags = super::common::convert_and_limit(&found_tags, query.limit);
    let marker = super::common::create_encoded_marker(&query, found_tags.len())?;

    let output = ListRoleTagsOutput::builder()
        .set_tags(tags)
        .set_is_truncated(marker.as_ref().map(|_v| true))
        .set_marker(marker)
        .build()
        .unwrap();
    Ok(output)
}

pub(crate) async fn list_roles(
    ctx: &OperationCtx, input: &ListRolesRequest, db: &LocalDb,
) -> Result<ListRolesOutput, OperationError> {
    input.validate("$")?;

    let query = input.into();

    // obtain connection
    let mut connection = db.new_connection().await?;

    let found_roles: Vec<SelectRole> = db::role::list(connection.as_mut(), ctx.account_id, &query).await?;
    let marker = super::common::create_encoded_marker(&query, found_roles.len())?;

    let mut roles: Vec<Role> = vec![];
    for i in 0..query.limit {
        let found_role = found_roles.get(i as usize);
        match found_role {
            None => break,
            Some(select_role) => {
                let role = Role::builder()
                    .role_id(&select_role.role_id)
                    .assume_role_policy_document(&select_role.assume_role_policy_document)
                    .role_name(&select_role.role_name)
                    .path(&select_role.path)
                    .arn(&select_role.arn)
                    .set_description(select_role.description.as_ref().map(|s| s.to_owned()))
                    .create_date(DateTime::from_secs(select_role.create_date))
                    .build()
                    .unwrap();
                roles.push(role);
            }
        }
    }

    let output = ListRolesOutput::builder()
        .set_roles(Some(roles))
        .set_is_truncated(marker.as_ref().map(|_v| true))
        .set_marker(marker)
        .build()
        .unwrap();
    Ok(output)
}

pub(crate) async fn list_role_policies(
    ctx: &OperationCtx, input: &ListRolePoliciesRequest, db: &LocalDb,
) -> Result<ListRolePoliciesOutput, OperationError> {
    input.validate("$")?;

    let mut connection = db.new_connection().await?;

    let role_name = input.role_name().unwrap().trim();
    let role_id = find_id_by_name(connection.as_mut(), ctx.account_id, role_name).await?;

    let query = ListInlinePoliciesQuery::new(role_id, input.max_items(), input.marker_type());
    let found_policies = db::role_inline_policy::find_by_role_id(connection.as_mut(), &query).await?;

    let policy_names = super::common::convert_and_limit(&found_policies, query.limit);
    let marker = super::common::create_encoded_marker(&query, found_policies.len())?;

    let output = ListRolePoliciesOutput::builder()
        .set_policy_names(policy_names)
        .set_is_truncated(marker.as_ref().map(|_v| true))
        .set_marker(marker)
        .build()
        .unwrap();
    Ok(output)
}

pub(crate) async fn tag_role(
    ctx: &OperationCtx, input: &TagRoleRequest, db: &LocalDb,
) -> Result<TagRoleOutput, OperationError> {
    input.validate("$")?;

    let mut tx = db.new_tx().await?;

    let role_id = find_id_by_name(tx.as_mut(), ctx.account_id, input.role_name().unwrap().trim()).await?;
    let mut role_tags = super::tag::prepare_for_db(input.tags(), role_id);

    db::Tags::Role.save_all(&mut tx, &mut role_tags).await?;
    let count = db::Tags::Role.count(tx.as_mut(), role_id).await?;
    if count > constants::tag::MAX_COUNT {
        return Err(OperationError::new(
            ApiErrorKind::LimitExceeded,
            format!("Cannot assign more than {} tags to IAM role.", constants::tag::MAX_COUNT).as_str(),
        ));
    }

    let output = TagRoleOutput::builder().build();

    tx.commit().await?;

    Ok(output)
}

pub(crate) async fn get_role_policy(
    ctx: &OperationCtx, input: &GetRolePolicyRequest, db: &LocalDb,
) -> Result<GetRolePolicyOutput, OperationError> {
    input.validate("$")?;

    let mut connection = db.new_connection().await?;

    let role_name = input.role_name().unwrap().trim();
    let role_id = find_id_by_name(connection.as_mut(), ctx.account_id, role_name).await?;

    let policy_name = input.policy_name().unwrap().trim();
    let inline_policy =
        db::role_inline_policy::find_by_role_id_and_name(connection.as_mut(), role_id, policy_name).await?;

    match inline_policy {
        None => Err(OperationError::new(
            ApiErrorKind::NoSuchEntity,
            format!("IAM inline policy with name '{policy_name}' not found for role with name '{role_name}'.").as_str(),
        )),
        Some(policy) => {
            let output = GetRolePolicyOutput::builder()
                .role_name(role_name)
                .policy_name(&policy.policy_name)
                .policy_document(&policy.policy_document)
                .build()
                .unwrap();
            Ok(output)
        }
    }
}

pub(crate) async fn put_role_policy(
    ctx: &OperationCtx, input: &PutRolePolicyRequest, db: &LocalDb,
) -> Result<PutRolePolicyOutput, OperationError> {
    input.validate("$")?;

    let mut tx = db.new_tx().await?;

    let role_id = find_id_by_name(tx.as_mut(), ctx.account_id, input.role_name().unwrap().trim()).await?;

    let mut inline_policy =
        DbInlinePolicy::new(role_id, input.policy_name().unwrap(), input.policy_document().unwrap());

    db::role_inline_policy::save(&mut tx, &mut inline_policy).await?;

    let output = PutRolePolicyOutput::builder().build();

    tx.commit().await?;
    Ok(output)
}

pub(crate) async fn untag_role(
    ctx: &OperationCtx, input: &UntagRoleRequest, db: &LocalDb,
) -> Result<UntagRoleOutput, OperationError> {
    input.validate("$")?;

    let mut tx = db.new_tx().await?;

    let role_id = find_id_by_name(tx.as_mut(), ctx.account_id, input.role_name().unwrap().trim()).await?;

    db::Tags::Role.delete_all(&mut tx, role_id, &input.tag_keys()).await?;

    let output = UntagRoleOutput::builder().build();

    tx.commit().await?;

    Ok(output)
}
