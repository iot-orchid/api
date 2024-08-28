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
                    .table(Cluster::Table)
                    .if_not_exists()
                    .col(
                        pk_auto(Cluster::Id)
                            .not_null()
                            .unique_key()
                            .auto_increment(),
                    )
                    .col(string(Cluster::Name).not_null())
                    .col(
                        timestamp_with_time_zone(Cluster::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Cluster::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Cluster::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Cluster {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}
