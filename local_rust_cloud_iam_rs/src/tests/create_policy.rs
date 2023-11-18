use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_iam::{config::Region, types::Tag};

use super::*;

#[actix_rt::test]
async fn create_policy() {
    let mut ctx = TEST_SUITE.create_test_ctx().await;
    let port = ctx.port;
    let config = aws_config::SdkConfig::builder()
        .region(Some(Region::new("eu-local-1")))
        .endpoint_url(format!("http://localhost:{}/iam", port))
        .credentials_provider(SharedCredentialsProvider::new(credentials_provider()))
        .build();
    let client = aws_sdk_iam::Client::new(&config);

    let response = client
        .create_policy()
        .description("policy-description")
        .path("/")
        .policy_document(include_str!("resources/create_user__permissions_boundary.json"))
        .policy_name("some-policy-name")
        .tags(Tag::builder().key("key1").value("value1").build())
        .tags(Tag::builder().key("key2").value("value2").build())
        .tags(Tag::builder().key("key3").value("value3").build())
        .send()
        .await
        .expect("Failed to create IAM policy");

    assert!(response.policy().is_some());
    assert!(response.policy().unwrap().tags().unwrap().len() > 0);

    ctx.stop_server().await;
}
