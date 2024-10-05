use super::common::parse_uuid;
use super::error::{Error, Result};
use super::ModelManager;
use crate::context::Ctx;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use entity::cluster;
use entity::user_cluster;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::{entity::prelude::*, QueryTrait};
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum ClusterUuid {
    Multiple(Vec<String>),
    Single(String),
}

#[derive(Deserialize, utoipa::ToSchema)]
#[allow(dead_code)]
pub struct ClusterCreate {
    #[schema(example = "factory-a")]
    name: String,
    #[schema(example = "us-west-1")]
    region: String,
    #[schema(example = "Cluster of sensors in factory-a used for telemetry.")]
    description: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ClusterRecord {
    #[schema(example = "<base64 encoded cluster uuid>")]
    pub uuid: Option<String>,
    #[schema(example = "factory-a")]
    pub name: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ClusterQuery {
    uuid: Option<String>,
}

pub struct ClusterBaseModelController {}

impl Into<ClusterUuid> for String {
    fn into(self) -> ClusterUuid {
        ClusterUuid::Single(self)
    }
}

impl Into<ClusterUuid> for Vec<String> {
    fn into(self) -> ClusterUuid {
        ClusterUuid::Multiple(self)
    }
}

impl ClusterBaseModelController {
    pub async fn create_cluster(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster: ClusterCreate,
    ) -> Result<ClusterRecord> {
        let new_uuid = uuid::Uuid::new_v4();
        let ctx_uuid = parse_uuid(&ctx.uuid)?;

        let new_cluster = cluster::ActiveModel {
            id: Set(new_uuid),
            name: Set(cluster.name),
            ..Default::default()
        }
        .insert(&mm.db)
        .await?;

        let _ = user_cluster::ActiveModel {
            user_id: Set(ctx_uuid),
            cluster_id: Set(new_uuid),
        }
        .insert(&mm.db)
        .await?;

        Ok(ClusterRecord {
            uuid: Some(URL_SAFE.encode(new_cluster.id)),
            name: new_cluster.name,
        })
    }

    pub async fn get_cluster(
        mm: &ModelManager,
        ctx: &Ctx,
        params: &ClusterQuery,
    ) -> Result<Vec<ClusterRecord>> {
        let ctx_uuid = parse_uuid(&ctx.uuid)?;

        let query_uuid = match &params.uuid {
            Some(uuid) => Some(parse_uuid(uuid)?),
            None => None,
        };

        let cluster_entities = Self::find_clusters_by_user_uuid(ctx_uuid)
            .apply_if(query_uuid, |q, v| q.filter(cluster::Column::Id.eq(v)))
            .all(&mm.db)
            .await?;

        let records: Vec<ClusterRecord> = cluster_entities
            .into_iter()
            .map(|e| ClusterRecord {
                uuid: Some(URL_SAFE.encode(e.id)),
                name: e.name,
            })
            .collect();

        Ok(records)
    }

    pub async fn exists<T>(mm: &ModelManager, ctx: &Ctx, cluster_uuid: T) -> Result<bool>
    where
        T: Into<ClusterUuid>,
    {
        let ctx_uuid = parse_uuid(&ctx.uuid)?;

        match cluster_uuid.into() {
            ClusterUuid::Single(uuid) => {
                let cluster_uuid = parse_uuid(&uuid)?;

                let count = Self::find_clusters_by_user_uuid(ctx_uuid)
                    .filter(cluster::Column::Id.eq(cluster_uuid))
                    .count(&mm.db)
                    .await?;

                Ok::<bool, Error>(count > 0)
            }
            ClusterUuid::Multiple(uuids) => {
                let cluster_uuids: Vec<Uuid> = uuids
                    .iter()
                    .map(|uuid| parse_uuid(uuid))
                    .collect::<Result<Vec<Uuid>>>()?;

                let uuid_count: u64 = cluster_uuids.len() as u64;

                let query = Self::find_clusters_by_user_uuid(ctx_uuid);

                let count = cluster_uuids
                    .into_iter()
                    .fold(query, |q, v| q.filter(cluster::Column::Id.eq(v)))
                    .count(&mm.db)
                    .await?;

                Ok::<bool, Error>(count == uuid_count)
            }
        }
    }

    pub(crate) fn find_clusters_by_user_uuid(user_uuid: Uuid) -> Select<cluster::Entity> {
        cluster::Entity::find()
            .inner_join(user_cluster::Entity)
            .filter(user_cluster::Column::UserId.eq(user_uuid))
    }
}
