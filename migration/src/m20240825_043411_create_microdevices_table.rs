use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Microdevice::Table)
                    .if_not_exists()
                    .col(pk_auto(Microdevice::Id))
                    .col(string(Microdevice::Title))
                    .col(string(Microdevice::Text))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Microdevice::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Microdevice {
    Table,
    Id,
    Title,
    Text,
}
