use super::common::parse_uuid;
use super::error::Result;
use super::{cluster::ClusterBaseModelController as ClusterBMC, ModelManager};
use crate::context::Ctx;
use entity::{cluster, microdevice, user_cluster};
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::{entity::prelude::*, QueryTrait};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct MicrodeviceCreate {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct MicrodeviceRecord {
    name: String,
    description: String,
}

pub struct MicrodeviceBaseModelController {}

impl MicrodeviceBaseModelController {
    pub async fn get_microdevice(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_uuid: String,
    ) -> Result<Vec<MicrodeviceRecord>> {
        if !ClusterBMC::exists(mm, ctx, cluster_uuid.clone()).await? {
            return Ok(vec![]);
        }

        let microdevice = microdevice::Entity::find()
            .filter(microdevice::Column::ClusterId.eq(parse_uuid(&cluster_uuid)?))
            .all(&mm.db)
            .await?;

        Ok(microdevice
            .into_iter()
            .map(|m| MicrodeviceRecord {
                name: m.name,
                description: m.description,
            })
            .collect())
    }
}
