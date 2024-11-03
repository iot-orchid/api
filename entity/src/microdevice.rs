//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0-rc.5

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "microdevice")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub cluster_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub topics: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cluster::Entity",
        from = "Column::ClusterId",
        to = "super::cluster::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Cluster,
}

impl Related<super::cluster::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cluster.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
