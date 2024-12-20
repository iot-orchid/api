pub use sea_orm_migration::prelude::*;

mod m20240825_042151_create_clusters_table;
mod m20240825_043411_create_microdevices_table;
mod m20240831_050316_user_table;
mod m20240831_050622_user_cluster_table;
mod m20240910_224132_add_topics_to_microdevices;
mod m20241007_224722_seed_tables;
mod m20241009_032952_make_description_nullable;
mod m20241116_234725_create_telemetry_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240825_042151_create_clusters_table::Migration),
            Box::new(m20240825_043411_create_microdevices_table::Migration),
            Box::new(m20240831_050316_user_table::Migration),
            Box::new(m20240831_050622_user_cluster_table::Migration),
            Box::new(m20240910_224132_add_topics_to_microdevices::Migration),
            Box::new(m20241007_224722_seed_tables::Migration),
            Box::new(m20241009_032952_make_description_nullable::Migration),
            Box::new(m20241116_234725_create_telemetry_table::Migration),
        ]
    }
}
