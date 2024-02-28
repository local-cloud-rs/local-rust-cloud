use aws_sdk_iam::operation::change_password::ChangePasswordOutput;
use aws_sdk_iam::operation::create_login_profile::CreateLoginProfileOutput;
use aws_sdk_iam::operation::delete_login_profile::DeleteLoginProfileOutput;
use aws_sdk_iam::operation::get_login_profile::GetLoginProfileOutput;
use aws_sdk_iam::operation::update_login_profile::UpdateLoginProfileOutput;
use aws_sdk_iam::types::LoginProfile;
use aws_smithy_types::DateTime;
use chrono::Utc;

use local_cloud_db::LocalDb;
use local_cloud_validate::NamedValidator;

use crate::http::aws::iam::db;
use crate::http::aws::iam::db::types::login_profile::InsertLoginProfile;
use crate::http::aws::iam::operations::ctx::OperationCtx;
use crate::http::aws::iam::operations::error::OperationError;
use crate::http::aws::iam::types::change_password::ChangePasswordRequest;
use crate::http::aws::iam::types::create_login_profile::CreateLoginProfileRequest;
use crate::http::aws::iam::types::delete_login_profile::DeleteLoginProfileRequest;
use crate::http::aws::iam::types::get_login_profile::GetLoginProfileRequest;
use crate::http::aws::iam::types::update_login_profile::UpdateLoginProfileRequest;

pub(crate) async fn create_login_profile(
    ctx: &OperationCtx, input: &CreateLoginProfileRequest, db: &LocalDb,
) -> Result<CreateLoginProfileOutput, OperationError> {
    input.validate("$")?;
    let mut tx = db.new_tx().await?;

    let found_user = super::user::find_by_name(ctx, tx.as_mut(), input.user_name().unwrap().trim()).await?;

    let current_time = Utc::now().timestamp_millis();
    let password_hash = super::secure::password_hash(input.password().unwrap())?;
    let password_reset_required = input.password_reset_required().unwrap_or(false);
    let mut insert_login_profile = InsertLoginProfile {
        id: None,
        user_id: found_user.id,
        password_hash,
        password_reset_required,
        create_date: current_time,
    };
    db::login_profile::create(&mut tx, &mut insert_login_profile).await?;
    let login_profile = LoginProfile::builder()
        .create_date(DateTime::from_millis(current_time))
        .user_name(&found_user.username)
        .password_reset_required(insert_login_profile.password_reset_required)
        .build()
        .unwrap();

    let output = CreateLoginProfileOutput::builder().login_profile(login_profile).build();

    tx.commit().await?;
    Ok(output)
}

pub(crate) async fn change_password(
    _ctx: &OperationCtx, input: &ChangePasswordRequest, _db: &LocalDb,
) -> Result<ChangePasswordOutput, OperationError> {
    input.validate("$")?;
    // TODO:
    let output = ChangePasswordOutput::builder().build();
    Ok(output)
}

pub(crate) async fn update_login_profile(
    _ctx: &OperationCtx, input: &UpdateLoginProfileRequest, _db: &LocalDb,
) -> Result<UpdateLoginProfileOutput, OperationError> {
    input.validate("$")?;
    let output = UpdateLoginProfileOutput::builder().build();
    Ok(output)
}

pub(crate) async fn get_login_profile(
    _ctx: &OperationCtx, input: &GetLoginProfileRequest, _db: &LocalDb,
) -> Result<GetLoginProfileOutput, OperationError> {
    input.validate("$")?;
    let output = GetLoginProfileOutput::builder().build();
    Ok(output)
}

pub(crate) async fn delete_login_profile(
    _ctx: &OperationCtx, input: &DeleteLoginProfileRequest, _db: &LocalDb,
) -> Result<DeleteLoginProfileOutput, OperationError> {
    input.validate("$")?;
    let output = DeleteLoginProfileOutput::builder().build();
    Ok(output)
}
