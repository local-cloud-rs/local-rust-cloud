use sqlx::FromRow;

#[derive(Clone, FromRow, Debug)]
pub(crate) struct DbPolicyTag {
    pub(crate) id: Option<i64>,
    pub(crate) policy_id: i64,
    pub(crate) key: String,
    pub(crate) value: String,
}

impl DbPolicyTag {
    pub(crate) fn new(policy_id: i64, key: impl Into<String>, value: impl Into<String>) -> Self {
        DbPolicyTag {
            id: None,
            policy_id,
            key: key.into(),
            value: value.into(),
        }
    }
}

impl Into<aws_sdk_iam::types::Tag> for &DbPolicyTag {
    fn into(self) -> aws_sdk_iam::types::Tag {
        aws_sdk_iam::types::Tag::builder()
            .key(&self.key)
            .value(&self.value)
            .build()
            .unwrap()
    }
}
