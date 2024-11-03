use super::common::{parse_cluster_id, parse_microdevice_id};
#[allow(unused_imports)]
use super::error::{Error, Result};
use super::{cluster::ClusterBaseModelController as ClusterBMC, ModelManager};
use crate::context::Ctx;
use entity::microdevice;
use sea_orm::ActiveValue::Set;
use sea_orm::{entity::prelude::*, QueryTrait};
use sea_orm::{EntityTrait, QuerySelect, SelectColumns};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct MicrodeviceCreate {
    #[schema(example = "sensor-1")]
    name: String,
    #[schema(example = "Sensor 1 in factory-a")]
    description: Option<String>,
    #[schema(example = json!([{
        "topic": "/temperature",
        "qos": 1,
        "name": "AHT10 temperature stream"
    }]))]
    topics: Option<Vec<MicrodeviceTopic>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct MicrodeviceTopic {
    #[schema(example = "/temperature")]
    pub topic: String,
    #[schema(example = "1")]
    pub qos: u8,
    #[schema(example = "AHT10 temperature stream")]
    pub name: String,
}

#[derive(Serialize, utoipa::ToSchema, sea_orm::FromQueryResult)]
pub struct MicrodeviceRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topics: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Deserialize, utoipa::ToSchema, Debug)]
pub struct MicrodeviceGetParams {
    name: Option<String>,
    id: Option<i32>,
    status: Option<DeviceStatus>,
    #[schema(example = true)]
    include_topics: Option<bool>,
    #[schema(example = true)]
    include_description: Option<bool>,
    #[schema(example = true)]
    include_cluster_uuid: Option<bool>,
}

impl MicrodeviceGetParams {
    pub fn for_action(microdevice_id: i32) -> Self {
        Self {
            name: None,
            id: Some(microdevice_id),
            status: None,
            include_topics: Some(true),
            include_description: Some(false),
            include_cluster_uuid: Some(true),
        }
    }
}

#[derive(Deserialize, utoipa::ToSchema, Debug)]
pub struct MicrodeviceDeleteParams {
    name: Option<String>,
    id: Option<i32>,
}

#[derive(Deserialize, utoipa::ToSchema, Debug)]
pub struct MicrodeviceUpdateParams {
    cluster_id: Option<String>,
    description: Option<String>,
    name: Option<String>,
    topics: Option<Vec<String>>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct DeviceRecord {
    uuid: String,
    name: String,
}
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}
pub struct MicrodeviceBaseModelController {}

impl MicrodeviceBaseModelController {
    pub async fn execute_action(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_uuid: String,
        microdevice_id: i32,
    ) -> Result<()> {
        let microdevice_data = Self::get_microdevice(
            mm,
            ctx,
            cluster_uuid,
            MicrodeviceGetParams::for_action(microdevice_id),
        )
        .await?;

        if microdevice_data.is_empty() {
            return Err(Error {
                kind: super::error::ErrorKind::MicrodeviceNotFound,
                message: "failed to execute action because microdevice was not found".to_string(),
            });
        }

        todo!()
    }

    pub async fn get_microdevice(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_uuid: String,
        params: MicrodeviceGetParams,
    ) -> Result<Vec<MicrodeviceRecord>> {
        ClusterBMC::exists(mm, ctx, cluster_uuid.clone()).await?;

        let microdevice: Vec<MicrodeviceRecord> = microdevice::Entity::find()
            .select_only()
            .select_column(microdevice::Column::Id)
            .select_column(microdevice::Column::Name)
            .filter(microdevice::Column::ClusterId.eq(parse_cluster_id(&cluster_uuid)?))
            .apply_if(params.id, |q, id| q.filter(microdevice::Column::Id.eq(id)))
            .apply_if(params.name, |q, name| {
                q.filter(microdevice::Column::Name.eq(name))
            })
            .apply_if(params.include_description, |q, include_description| {
                if include_description {
                    return q.select_column(microdevice::Column::Description);
                }
                q
            })
            .apply_if(params.include_topics, |q, include_topics| {
                if include_topics {
                    return q.select_column(microdevice::Column::Topics);
                }
                q
            })
            .apply_if(params.include_cluster_uuid, |q, include_cluster_uuid| {
                if include_cluster_uuid {
                    return q.select_column(microdevice::Column::ClusterId);
                }
                q
            })
            .into_model()
            .all(&mm.db)
            .await?;

        Ok(microdevice)
    }

    pub async fn create_microdevice(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_uuid: String,
        microdevice: MicrodeviceCreate,
    ) -> Result<MicrodeviceRecord> {
        ClusterBMC::exists(mm, ctx, cluster_uuid.clone()).await?;

        let mut new_microdevice = microdevice::ActiveModel {
            name: Set(microdevice.name),
            cluster_id: Set(parse_cluster_id(&cluster_uuid)?),
            ..Default::default()
        };

        microdevice.topics.map(|v| {
            new_microdevice.topics = Set(Some(serde_json::to_value(v).unwrap()));
        });

        microdevice.description.map(|v| {
            new_microdevice.description = Set(Some(v));
        });

        let new_microdevice = new_microdevice.insert(&mm.db).await?;

        Ok(MicrodeviceRecord {
            cluster_id: Some(new_microdevice.cluster_id),
            id: Some(new_microdevice.id),
            name: Some(new_microdevice.name),
            description: new_microdevice.description,
            topics: new_microdevice.topics,
        })
    }

    #[allow(unused_variables)]
    pub async fn delete_microdevice(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_uuid: String,
        params: MicrodeviceDeleteParams,
    ) -> Result<()> {
        ClusterBMC::exists(mm, ctx, cluster_uuid.clone()).await?;

        let target = microdevice::Entity::find()
            .filter(microdevice::Column::ClusterId.eq(parse_cluster_id(&cluster_uuid)?))
            .apply_if(params.id, |q, v| q.filter(microdevice::Column::Id.eq(v)))
            .apply_if(params.name, |q, v| {
                q.filter(microdevice::Column::Name.eq(v))
            })
            .one(&mm.db)
            .await?;

        match target {
            Some(target) => {
                target.delete(&mm.db).await?;
            }
            None => {
                return Err(Error {
                    kind: super::error::ErrorKind::MicrodeviceNotFound,
                    message: "failed to delete microdevice because it was not found".to_string(),
                })
            }
        }

        Ok(())
    }

    #[allow(unused_variables)]
    pub async fn update_microdevice(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_uuid: String,
        microdevice_id: i32,
        params: MicrodeviceUpdateParams,
    ) -> Result<MicrodeviceRecord> {
        ClusterBMC::exists(mm, ctx, cluster_uuid.clone()).await?;

        let target = microdevice::Entity::find()
            .filter(microdevice::Column::ClusterId.eq(parse_cluster_id(&cluster_uuid)?))
            .filter(microdevice::Column::Id.eq(parse_microdevice_id(microdevice_id)?))
            .one(&mm.db)
            .await?;

        match target {
            Some(target) => {
                let mut update = microdevice::ActiveModel::from(target);

                if let Some(new_cluster_id) = params.cluster_id {
                    ClusterBMC::exists(mm, ctx, new_cluster_id.to_string()).await?;
                    update.cluster_id = Set(parse_cluster_id(&new_cluster_id)?);
                }

                params
                    .description
                    .map(|v| update.description = Set(Some(v)));

                params.name.map(|v| update.name = Set(v));

                params
                    .topics
                    .map(|v| update.topics = Set(Some(serde_json::to_value(v).unwrap())));

                let res = update.update(&mm.db).await?;

                return Ok(MicrodeviceRecord {
                    cluster_id: Some(res.cluster_id),
                    id: Some(res.id),
                    name: Some(res.name),
                    description: res.description,
                    topics: res.topics,
                });
            }
            None => {
                return Err(Error {
                    kind: super::error::ErrorKind::MicrodeviceNotFound,
                    message: "failed to update microdevice because it was not found".to_string(),
                })
            }
        }
    }
}
