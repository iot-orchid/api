use super::common::parse_cluster_id;
use super::error::{Error, Result};
use super::ModelManager;
use crate::context::Ctx;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use entity::{cluster, user};
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
pub struct ClusterCreate {
    #[schema(example = "factory-a")]
    name: String,
    #[schema(example = "us-west-1")]
    region: String,
    #[schema(example = "Cluster of sensors in factory-a used for telemetry.")]
    description: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ClusterDelete {
    #[schema(example = "<base64 encoded cluster uuid>")]
    id: Option<String>,
    #[schema(example = "<base64 encoded cluster uuid>, <base64 encoded cluster uuid>, ...")]
    ids: Option<Vec<String>>,
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
    fn validate_user_ctx(ctx: &Ctx) -> Result<&str> {
        if let Ctx::UserCtx { user_id, .. } = ctx {
            Ok(user_id)
        } else {
            Err(Error {
                kind: super::error::ErrorKind::UnauthorizedClusterAccess,
                message: "Microdevice context cannot access cluster operations".to_string(),
            })
        }
    }

    pub async fn create_cluster(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster: ClusterCreate,
    ) -> Result<ClusterRecord> {
        let new_uuid = uuid::Uuid::new_v4();
        let user_id = Self::validate_user_ctx(ctx)?;
        let ctx_uuid = parse_cluster_id(&user_id.into())?;

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
        let user_id = Self::validate_user_ctx(ctx)?;
        let ctx_uuid = parse_cluster_id(&user_id.into())?;

        let query_uuid = match &params.uuid {
            Some(uuid) => Some(parse_cluster_id(uuid)?),
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

    pub async fn exists<T>(mm: &ModelManager, ctx: &Ctx, cluster_uuid: T) -> Result<()>
    where
        T: Into<ClusterUuid>,
    {
        let user_id = Self::validate_user_ctx(ctx)?;
        let ctx_uuid = parse_cluster_id(&user_id.into())?;

        match cluster_uuid.into() {
            ClusterUuid::Single(uuid) => {
                let cluster_uuid = parse_cluster_id(&uuid)?;

                let count = Self::find_clusters_by_user_uuid(ctx_uuid)
                    .filter(cluster::Column::Id.eq(cluster_uuid))
                    .count(&mm.db)
                    .await?;

                if count == 0 {
                    return Err(Error {
                        kind: super::error::ErrorKind::ClusterNotFound,
                        message: format!("cluster `{}` not found.", cluster_uuid),
                    });
                }

                Ok(())
            }
            ClusterUuid::Multiple(uuids) => {
                let cluster_uuids: Vec<Uuid> = uuids
                    .iter()
                    .map(|uuid| parse_cluster_id(uuid))
                    .collect::<Result<Vec<Uuid>>>()?;

                let uuid_count: u64 = cluster_uuids.len() as u64;

                let query = Self::find_clusters_by_user_uuid(ctx_uuid);

                let count = cluster_uuids
                    .into_iter()
                    .fold(query, |q, v| q.filter(cluster::Column::Id.eq(v)))
                    .count(&mm.db)
                    .await?;

                if count != uuid_count {
                    return Err(Error {
                        kind: super::error::ErrorKind::ClusterNotFound,
                        message: "One or more clusters not found".to_string(),
                    });
                }

                Ok(())
            }
        }
    }

    pub async fn delete_cluster(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster: ClusterDelete,
    ) -> Result<u64> {
        let user_id = Self::validate_user_ctx(ctx)?;
        let ctx_uuid = parse_cluster_id(&user_id.into())?;

        let cluster_uuids = match cluster {
            ClusterDelete {
                id: Some(uuid),
                ids: None,
            } => {
                vec![parse_cluster_id(&uuid)?]
            }

            ClusterDelete {
                id: None,
                ids: Some(uuids),
            } => uuids
                .iter()
                .map(|uuid| parse_cluster_id(uuid))
                .collect::<Result<Vec<Uuid>>>()?,

            _ => {
                return Err(Error {
                    kind: super::error::ErrorKind::InvalidContext,
                    message: "Invalid context for cluster deletion".to_string(),
                });
            }
        };

        let res = user_cluster::Entity::delete_many()
            .filter(user_cluster::Column::UserId.eq(ctx_uuid))
            .filter(user_cluster::Column::ClusterId.is_in(cluster_uuids))
            .exec(&mm.db).await?;

        Ok(res.rows_affected)
    }

    pub(crate) fn find_clusters_by_user_uuid(user_uuid: Uuid) -> Select<cluster::Entity> {
        cluster::Entity::find()
            .inner_join(user_cluster::Entity)
            .filter(user_cluster::Column::UserId.eq(user_uuid))
    }
}
