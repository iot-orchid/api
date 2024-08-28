use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240825_042151_create_clusters_table::Cluster;

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
                    .col(integer(Microdevice::ClusterID).not_null())
                    .col(string(Microdevice::Name))
                    .col(string(Microdevice::Description))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_microdevice_cluster_id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .from(Microdevice::Table, Microdevice::ClusterID)
                            .to(Cluster::Table, Cluster::Id),
                    )
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
pub enum Microdevice {
    Table,
    Id,
    ClusterID,
    Name,
    Description,
}
