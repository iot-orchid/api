use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240825_043411_create_microdevices_table::Microdevice;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(TelemetryRecord::Table)
                    .if_not_exists()
                    .col(
                        timestamp(TelemetryRecord::Timestamp)
                            .not_null()
                            .primary_key(),
                    )
                    .col(integer(TelemetryRecord::MicrodeviceId).not_null())
                    .col(string(TelemetryRecord::SourceTopic).not_null())
                    .col(string(TelemetryRecord::SourceName).not_null())
                    .col(json_binary(TelemetryRecord::Data).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TelemetryRecord::Table, TelemetryRecord::MicrodeviceId)
                            .to(Microdevice::Table, Microdevice::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(TelemetryRecord::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TelemetryRecord {
    Table,
    MicrodeviceId,
    SourceTopic,
    SourceName,
    Description,
    Timestamp,
    Data,
}
