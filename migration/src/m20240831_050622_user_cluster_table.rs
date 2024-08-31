use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240825_042151_create_clusters_table::Cluster;
use crate::m20240831_050316_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserCluster::Table)
                    .if_not_exists()
                    .col(uuid(UserCluster::UserId).not_null())
                    .col(uuid(UserCluster::ClusterId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_cluster_user_id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .from(UserCluster::Table, UserCluster::UserId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_cluster_cluster_id")
                            .on_delete(ForeignKeyAction::Cascade)
                            .from(UserCluster::Table, UserCluster::ClusterId)
                            .to(Cluster::Table, Cluster::Id),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_user_cluster")
                            .col(UserCluster::UserId)
                            .col(UserCluster::ClusterId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserCluster::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserCluster {
    Table,
    UserId,
    ClusterId,
}
