use sqlx::sqlite::SqliteRow;
use sqlx::{Error, Executor, FromRow, QueryBuilder, Row, Sqlite, Transaction};

use crate::http::aws::iam::db::types::common::Pageable;
use crate::http::aws::iam::db::types::group::{
    InsertGroup, ListGroupsByUserQuery, ListGroupsQuery, SelectGroup, UpdateGroupQuery,
};

pub(crate) async fn create<'a>(tx: &mut Transaction<'a, Sqlite>, group: &mut InsertGroup) -> Result<(), Error> {
    let result = sqlx::query(
        r#"INSERT INTO groups (
                    account_id,
                    group_name,
                    unique_group_name,
                    arn,
                    path,
                    group_id,
                    create_date
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id"#,
    )
    .bind(group.account_id)
    .bind(&group.group_name)
    .bind(&group.group_name.to_uppercase())
    .bind(&group.arn)
    .bind(&group.path)
    .bind(&group.group_id)
    .bind(group.create_date)
    .map(|row: SqliteRow| row.get::<i64, &str>("id"))
    .fetch_one(tx.as_mut())
    .await?;
    group.id = Some(result);
    Ok(())
}

pub(crate) async fn list<'a, E>(
    executor: E, account_id: i64, query: &ListGroupsQuery,
) -> Result<Vec<SelectGroup>, Error>
where
    E: 'a + Executor<'a, Database = Sqlite>,
{
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"
            SELECT 
                id,
                account_id,
                group_name,
                unique_group_name,
                arn,
                path,
                group_id,
                create_date
            FROM groups
            WHERE path LIKE "#,
    );
    let result = query_builder
        .push_bind(format!("{}%", &query.path_prefix))
        .push(" AND account_id = ")
        .push_bind(account_id)
        .push(" ORDER BY id ASC")
        .push(" LIMIT ")
        .push_bind(query.limit + 1) // request more elements than we need to return. used to identify if NextPage token needs to be generated
        .push(" OFFSET ")
        .push_bind(query.skip)
        .build()
        .map(|row: SqliteRow| SelectGroup::from_row(&row).unwrap())
        .fetch_all(executor)
        .await?;

    Ok(result)
}

pub(crate) async fn find_by_name<'a, E>(
    executor: E, account_id: i64, group_name: &str,
) -> Result<Option<SelectGroup>, Error>
where
    E: 'a + Executor<'a, Database = Sqlite>,
{
    let group = sqlx::query(
        r#"
            SELECT 
                id,
                account_id,
                group_name,
                unique_group_name,
                arn,
                path,
                group_id,
                create_date
            FROM groups
            WHERE account_id = $1 AND unique_group_name = $2
    "#,
    )
    .bind(account_id)
    .bind(group_name.to_uppercase())
    .map(|row: SqliteRow| SelectGroup::from_row(&row).unwrap())
    .fetch_optional(executor)
    .await?;

    Ok(group)
}

pub(crate) async fn find_id_by_name<'a, E>(executor: E, account_id: i64, group_name: &str) -> Result<Option<i64>, Error>
where
    E: 'a + Executor<'a, Database = Sqlite>,
{
    let group_id = sqlx::query(
        r#"
            SELECT 
                id
            FROM groups
            WHERE account_id = $1 AND unique_group_name = $2
    "#,
    )
    .bind(account_id)
    .bind(group_name.to_uppercase())
    .map(|row: SqliteRow| row.get::<i64, &str>("id"))
    .fetch_optional(executor)
    .await?;

    Ok(group_id)
}

pub(crate) async fn assign_user_to_group<'a>(
    tx: &mut Transaction<'a, Sqlite>, group_id: i64, user_id: i64,
) -> Result<(), Error> {
    sqlx::query(r#"INSERT INTO group_users (group_id, user_id) VALUES ($1, $2)"#)
        .bind(group_id)
        .bind(user_id)
        .execute(tx.as_mut())
        .await?;
    Ok(())
}

pub(crate) async fn assign_policy_to_group<'a>(
    tx: &mut Transaction<'a, Sqlite>, group_id: i64, policy_id: i64,
) -> Result<(), Error> {
    sqlx::query(r#"INSERT INTO policy_groups (group_id, policy_id) VALUES ($1, $2)"#)
        .bind(group_id)
        .bind(policy_id)
        .execute(tx.as_mut())
        .await?;
    Ok(())
}

pub(crate) async fn find_by_user_id<'a, E>(
    executor: E, query: &ListGroupsByUserQuery,
) -> Result<Vec<SelectGroup>, Error>
where
    E: 'a + Executor<'a, Database = Sqlite>,
{
    let groups = sqlx::query(
        r#"
            SELECT 
                g.id AS id,
                g.account_id AS account_id,
                g.group_name AS group_name,
                g.unique_group_name AS unique_group_name,
                g.arn AS arn,
                g.path AS path,
                g.group_id AS group_id,
                g.create_date AS create_date
            FROM users u 
            LEFT JOIN group_users gu ON u.id = gu.user_id 
            LEFT JOIN groups g ON gu.group_id = g.id 
            WHERE u.id = $1 ORDER BY g.unique_group_name
            LIMIT $2 OFFSET $3
    "#,
    )
    .bind(query.user_id)
    .bind(query.limit() + 1)
    .bind(query.skip())
    .map(|row: SqliteRow| SelectGroup::from_row(&row).unwrap())
    .fetch_all(executor)
    .await?;

    Ok(groups)
}

pub(crate) async fn update<'a, E>(executor: E, account_id: i64, query: &UpdateGroupQuery) -> Result<bool, Error>
where
    E: 'a + Executor<'a, Database = Sqlite>,
{
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE groups SET");
    let mut added = false;
    if let Some(new_group_name) = &query.new_group_name {
        query_builder
            .push(" group_name=")
            .push_bind(new_group_name)
            .push(" , unique_group_name=")
            .push_bind(new_group_name.to_uppercase());
        added = true;
    }
    if let Some(new_path) = &query.new_path {
        if added {
            query_builder.push(" ,");
        }
        query_builder.push(" path=").push_bind(new_path);
    }

    let result = query_builder
        .push(" WHERE account_id=")
        .push_bind(account_id)
        .push(" AND unique_group_name=")
        .push_bind(&query.group_name.to_uppercase())
        .build()
        .execute(executor)
        .await?;

    Ok(result.rows_affected() == 1)
}
