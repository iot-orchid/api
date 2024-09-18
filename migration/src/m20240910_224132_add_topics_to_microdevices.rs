#![allow(unused_imports)]
use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240825_043411_create_microdevices_table::Microdevice::Table as MD_TABLE;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(MD_TABLE)
                    .add_column(ColumnDef::new(Microdevice::Topics).json())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MD_TABLE)
                    .drop_column(Microdevice::Topics)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
#[allow(dead_code)]
enum Microdevice {
    Table,
    Id,
    Title,
    Text,
    Topics,
}
