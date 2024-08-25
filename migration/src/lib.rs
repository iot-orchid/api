pub use sea_orm_migration::prelude::*;

mod m20240825_042151_create_clusters_table;
mod m20240825_043411_create_microdevices_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240825_042151_create_clusters_table::Migration),
            Box::new(m20240825_043411_create_microdevices_table::Migration),
        ]
    }
}
