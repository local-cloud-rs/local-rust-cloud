use aws_sdk_iam::operation::add_role_to_instance_profile::AddRoleToInstanceProfileOutput;
use aws_sdk_iam::operation::create_instance_profile::CreateInstanceProfileOutput;
use aws_sdk_iam::operation::delete_instance_profile::DeleteInstanceProfileOutput;
use aws_sdk_iam::operation::get_instance_profile::GetInstanceProfileOutput;
use aws_sdk_iam::operation::list_instance_profile_tags::ListInstanceProfileTagsOutput;
use aws_sdk_iam::operation::list_instance_profiles::ListInstanceProfilesOutput;
use aws_sdk_iam::operation::list_instance_profiles_for_role::ListInstanceProfilesForRoleOutput;
use aws_sdk_iam::operation::remove_role_from_instance_profile::RemoveRoleFromInstanceProfileOutput;
use aws_sdk_iam::operation::tag_instance_profile::TagInstanceProfileOutput;
use aws_sdk_iam::operation::untag_instance_profile::UntagInstanceProfileOutput;
use aws_sdk_iam::types::InstanceProfile;
use aws_smithy_types::DateTime;
use chrono::Utc;
use sqlx::{Executor, Sqlite};

use local_cloud_db::LocalDb;
use local_cloud_validate::NamedValidator;

use crate::http::aws::iam::actions::error::ApiErrorKind;
use crate::http::aws::iam::db::types::instance_profile::InsertInstanceProfile;
use crate::http::aws::iam::db::types::resource_identifier::ResourceType;
use crate::http::aws::iam::db::types::tags::ListTagsQuery;
use crate::http::aws::iam::operations::common::create_resource_id;
use crate::http::aws::iam::operations::ctx::OperationCtx;
use crate::http::aws::iam::operations::error::OperationError;
use crate::http::aws::iam::types::add_role_to_instance_profile::AddRoleToInstanceProfileRequest;
use crate::http::aws::iam::types::create_instance_profile::CreateInstanceProfileRequest;
use crate::http::aws::iam::types::delete_instance_profile::DeleteInstanceProfileRequest;
use crate::http::aws::iam::types::get_instance_profile::GetInstanceProfileRequest;
use crate::http::aws::iam::types::list_instance_profile_tags::ListInstanceProfileTagsRequest;
use crate::http::aws::iam::types::list_instance_profiles::ListInstanceProfilesRequest;
use crate::http::aws::iam::types::list_instance_profiles_for_role::ListInstanceProfilesForRoleRequest;
use crate::http::aws::iam::types::remove_role_from_instance_profile::RemoveRoleFromInstanceProfileRequest;
use crate::http::aws::iam::types::tag_instance_profile::TagInstanceProfileRequest;
use crate::http::aws::iam::types::untag_instance_profile::UntagInstanceProfileRequest;
use crate::http::aws::iam::{constants, db};

pub(crate) async fn create_instance_profile(
    ctx: &OperationCtx, input: &CreateInstanceProfileRequest, db: &LocalDb,
) -> Result<CreateInstanceProfileOutput, OperationError> {
    input.validate("$")?;

    let current_time = Utc::now().timestamp();
    let mut tx = db.new_tx().await?;

    let path = input.path().unwrap_or("/");
    let instance_profile_name = input.instance_profile_name().unwrap();
    let arn = format!("arn:aws:iam::{:0>12}:instance-profile{}{}", ctx.account_id, path, instance_profile_name);
    let instance_profile_id =
        create_resource_id(&mut tx, constants::instance_profile::PREFIX, ResourceType::InstanceProfile).await?;

    let mut insert_instance_profile = InsertInstanceProfile {
        id: None,
        account_id: ctx.account_id,
        instance_profile_name: instance_profile_name.to_owned(),
        instance_profile_id,
        arn,
        path: path.to_owned(),
        create_date: current_time,
    };

    db::instance_profile::create(&mut tx, &mut insert_instance_profile).await?;

    let mut tags = super::tag::prepare_for_db(input.tags(), insert_instance_profile.id.unwrap());
    db::Tags::InstanceProfile.save_all(&mut tx, &mut tags).await?;

    let instance_profile = InstanceProfile::builder()
        .instance_profile_name(&insert_instance_profile.instance_profile_name)
        .instance_profile_id(&insert_instance_profile.instance_profile_id)
        .path(&insert_instance_profile.path)
        .arn(&insert_instance_profile.arn)
        .create_date(DateTime::from_secs(current_time))
        .set_roles(Some(Vec::with_capacity(0))) // no roles when just create an instance profile.
        .set_tags(super::tag::prepare_for_output(&tags))
        .build()
        .unwrap();

    let output = CreateInstanceProfileOutput::builder()
        .instance_profile(instance_profile)
        .build();

    tx.commit().await?;
    Ok(output)
}

pub(crate) async fn find_id_by_name<'a, E>(
    ctx: &OperationCtx, executor: E, instance_profile_name: &str,
) -> Result<i64, OperationError>
where
    E: 'a + Executor<'a, Database = Sqlite>,
{
    match db::instance_profile::find_id_by_name(executor, ctx.account_id, instance_profile_name).await? {
        Some(id) => Ok(id),
        None => {
            return Err(OperationError::new(
                ApiErrorKind::NoSuchEntity,
                format!("IAM instance profile with name '{}' doesn't exist.", instance_profile_name).as_str(),
            ));
        }
    }
}

pub(crate) async fn add_role_to_instance_profile(
    ctx: &OperationCtx, input: &AddRoleToInstanceProfileRequest, db: &LocalDb,
) -> Result<AddRoleToInstanceProfileOutput, OperationError> {
    input.validate("$")?;

    let mut tx = db.new_tx().await?;
    let instance_profile_id = find_id_by_name(ctx, tx.as_mut(), input.instance_profile_name().unwrap().trim()).await?;
    let role_id = super::role::find_id_by_name(tx.as_mut(), ctx.account_id, input.role_name().unwrap().trim()).await?;

    db::instance_profile::assign_role_to_instance_profile(&mut tx, instance_profile_id, role_id).await?;

    let output = AddRoleToInstanceProfileOutput::builder().build();

    tx.commit().await?;
    Ok(output)
}

pub(crate) async fn tag_instance_profile(
    ctx: &OperationCtx, input: &TagInstanceProfileRequest, db: &LocalDb,
) -> Result<TagInstanceProfileOutput, OperationError> {
    input.validate("$")?;

    let mut tx = db.new_tx().await?;

    let instance_profile_id = find_id_by_name(ctx, tx.as_mut(), input.instance_profile_name().unwrap().trim()).await?;
    let mut instance_profile_tags = super::tag::prepare_for_db(input.tags(), instance_profile_id);

    db::Tags::InstanceProfile
        .save_all(&mut tx, &mut instance_profile_tags)
        .await?;
    let count = db::Tags::InstanceProfile
        .count(tx.as_mut(), instance_profile_id)
        .await?;
    if count > constants::tag::MAX_COUNT {
        return Err(OperationError::new(
            ApiErrorKind::LimitExceeded,
            format!("Cannot assign more than {} tags to IAM instance profile.", constants::tag::MAX_COUNT).as_str(),
        ));
    }

    let output = TagInstanceProfileOutput::builder().build();

    tx.commit().await?;

    Ok(output)
}

pub(crate) async fn untag_instance_profile(
    ctx: &OperationCtx, input: &UntagInstanceProfileRequest, db: &LocalDb,
) -> Result<UntagInstanceProfileOutput, OperationError> {
    input.validate("$")?;

    let mut tx = db.new_tx().await?;

    let instance_profile_id = find_id_by_name(ctx, tx.as_mut(), input.instance_profile_name().unwrap().trim()).await?;

    db::Tags::InstanceProfile
        .delete_all(&mut tx, instance_profile_id, &input.tag_keys())
        .await?;

    let output = UntagInstanceProfileOutput::builder().build();

    tx.commit().await?;

    Ok(output)
}

pub(crate) async fn list_instance_profile_tags(
    ctx: &OperationCtx, input: &ListInstanceProfileTagsRequest, db: &LocalDb,
) -> Result<ListInstanceProfileTagsOutput, OperationError> {
    input.validate("$")?;

    let mut connection = db.new_connection().await?;

    let found_instance_profile_id =
        find_id_by_name(ctx, connection.as_mut(), input.instance_profile_name().unwrap().trim()).await?;

    let query = ListTagsQuery::new(input.max_items(), input.marker_type());
    let found_tags = db::Tags::InstanceProfile
        .list(connection.as_mut(), found_instance_profile_id, &query)
        .await?;

    let tags = super::common::convert_and_limit(&found_tags, query.limit);
    let marker = super::common::create_encoded_marker(&query, found_tags.len())?;

    let output = ListInstanceProfileTagsOutput::builder()
        .set_tags(tags)
        .set_is_truncated(marker.as_ref().map(|_v| true))
        .set_marker(marker)
        .build()
        .unwrap();

    Ok(output)
}

pub(crate) async fn remove_role_from_instance_profile(
    ctx: &OperationCtx, input: &RemoveRoleFromInstanceProfileRequest, db: &LocalDb,
) -> Result<RemoveRoleFromInstanceProfileOutput, OperationError> {
    input.validate("$")?;

    let output = RemoveRoleFromInstanceProfileOutput::builder().build();
    Ok(output)
}

pub(crate) async fn delete_instance_profile(
    ctx: &OperationCtx, input: &DeleteInstanceProfileRequest, db: &LocalDb,
) -> Result<DeleteInstanceProfileOutput, OperationError> {
    input.validate("$")?;

    let output = DeleteInstanceProfileOutput::builder().build();
    Ok(output)
}

pub(crate) async fn get_instance_profile(
    ctx: &OperationCtx, input: &GetInstanceProfileRequest, db: &LocalDb,
) -> Result<GetInstanceProfileOutput, OperationError> {
    input.validate("$")?;

    let output = GetInstanceProfileOutput::builder().build();
    Ok(output)
}

pub(crate) async fn list_instance_profiles(
    ctx: &OperationCtx, input: &ListInstanceProfilesRequest, db: &LocalDb,
) -> Result<ListInstanceProfilesOutput, OperationError> {
    input.validate("$")?;

    let output = ListInstanceProfilesOutput::builder().build().unwrap();
    Ok(output)
}

pub(crate) async fn list_instance_profiles_for_role(
    ctx: &OperationCtx, input: &ListInstanceProfilesForRoleRequest, db: &LocalDb,
) -> Result<ListInstanceProfilesForRoleOutput, OperationError> {
    input.validate("$")?;

    let output = ListInstanceProfilesForRoleOutput::builder().build().unwrap();
    Ok(output)
}
