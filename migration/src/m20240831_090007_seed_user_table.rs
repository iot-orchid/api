use entity::user;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::entity::*;
use uuid;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let res = user::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            username: Set("foo".to_string()),
            password_hash: Set(bcrypt::hash("bar", bcrypt::DEFAULT_COST).unwrap()),
        }
        .insert(db)
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(DbErr::from(e)),
        }
    }
}
