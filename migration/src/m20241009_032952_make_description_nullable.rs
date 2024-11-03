#[allow(unused_imports)]
use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240825_043411_create_microdevices_table::Microdevice;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        
        manager.alter_table(
            Table::alter()
                .table(Microdevice::Table)
            .modify_column(ColumnDef::new(Microdevice::Description).string().null())
                .to_owned(),
        ).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
}

}
