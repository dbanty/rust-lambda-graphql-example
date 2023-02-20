use async_graphql::{EmptySubscription, Object, Result, Schema, SimpleObject};
use lazy_static::lazy_static;

#[derive(SimpleObject)]
struct User {
    id: i32,
    name: String,
}

pub(crate) struct Query;

#[Object]
impl Query {
    async fn users(&self) -> Result<Vec<User>> {
        let users: Vec<User> = vec![
            User {
                id: 1,
                name: "Alex".into(),
            },
            User {
                id: 2,
                name: "Jesse".into(),
            },
        ];
        tracing::info!("Finished query");
        Ok(users)
    }
}

pub(crate) struct Mutation;

#[Object]
impl Mutation {
    async fn create_user(&self, name: String) -> Result<User> {
        tracing::info!("User not created (no datasource)");
        Ok(User { id: 1, name })
    }
}

lazy_static! {
    pub(crate) static ref SCHEMA: Schema<Query, Mutation, EmptySubscription> =
        Schema::build(Query, Mutation, EmptySubscription).finish();
}

#[cfg(test)]
mod test_users {
    use serde_json::json;

    use super::SCHEMA;

    #[tokio::test]
    async fn test_create_and_get_user() {
        let res = SCHEMA
            .execute("mutation {createUser(name: \"Alex\") {name}}")
            .await;
        let data = res.data.into_json().expect("Result was not JSON");
        let name = data["createUser"]["name"]
            .as_str()
            .expect("Result was not string");
        assert_eq!(name, "Alex");
        let res = SCHEMA
            .execute("{users{id, name}}")
            .await
            .data
            .into_json()
            .unwrap();
        assert_eq!(
            res,
            json!({"users": [{"name": "Alex", "id": 1}, {"name": "Jesse", "id": 2}]})
        )
    }
}
