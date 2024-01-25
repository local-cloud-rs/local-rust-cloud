use aws_sdk_iam::operation::create_user::CreateUserOutput;

use local_cloud_db::LocalDb;

use crate::http::aws::iam;
use crate::http::aws::iam::actions::error::{ApiError, ApiErrorKind};
use crate::http::aws::iam::actions::wrapper::OutputWrapper;
use crate::http::aws::iam::operations::ctx::OperationCtx;
use crate::http::aws::iam::operations::error::OperationError;
use crate::http::aws::iam::types::create_user_request::CreateUserRequest;

impl CreateUserRequest {
    pub async fn execute(
        &self, account_id: i64, aws_request_id: &str, db: &LocalDb,
    ) -> Result<OutputWrapper<CreateUserOutput>, ApiError> {
        let ctx = OperationCtx::new(account_id, aws_request_id);
        let output = iam::operations::user::create_user(&ctx, self, db)
            .await
            .map_err(|error| match error {
                OperationError::Service { kind, msg } => {
                    if kind == ApiErrorKind::EntityAlreadyExists {
                        ApiError::new(
                            kind,
                            format!("User with name '{}' already exists.", self.user_name().unwrap().trim()).as_str(),
                            aws_request_id,
                        )
                    } else {
                        ApiError::new(kind, &msg, aws_request_id)
                    }
                }
                OperationError::Validation(error) => ApiError::from_validation_error(&error, aws_request_id),
            })?;
        Ok(OutputWrapper::new(output, aws_request_id))
    }
}
