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

#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MicrodeviceId {
    Id(i32),
    Name(String),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MicrodeviceAction {
    Start,
    Stop,
    Restart,
    Reset,
    PowerOn,
    PowerOff,
    #[serde(untagged)]
    UserDefined(String),
}

impl From<i32> for MicrodeviceId {
    fn from(id: i32) -> Self {
        Self::Id(id)
    }
}

impl From<String> for MicrodeviceId {
    fn from(name: String) -> Self {
        Self::Name(name)
    }
}

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

#[derive(Clone, Serialize, utoipa::ToSchema, sea_orm::FromQueryResult, Debug)]
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
    pub name: Option<String>,
    pub id: Option<i32>,
    // status: Option<DeviceStatus>,
    #[schema(example = true)]
    pub include_topics: Option<bool>,
    #[schema(example = true)]
    pub include_description: Option<bool>,
    #[schema(example = true)]
    pub include_cluster_id: Option<bool>,
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

#[derive(Serialize, utoipa::ToSchema)]
pub struct MicrodeviceActionResponse {
    microdevice_id: MicrodeviceId,
    status: String,
    message: String,
    payload: serde_json::Value,
}

#[derive(Serialize)]
pub struct MicrodeviceActionMessage {
    cluster_id: String,
    microdevice_id: MicrodeviceId,
    action: MicrodeviceAction,
    payload: serde_json::Value,
}

impl From<String> for MicrodeviceAction {
    fn from(action: String) -> Self {
        match action.as_str() {
            "start" => Self::Start,
            "stop" => Self::Stop,
            "restart" => Self::Restart,
            "reset" => Self::Reset,
            "power-on" => Self::PowerOn,
            "power-off" => Self::PowerOff,
            _ => Self::UserDefined(action),
        }
    }
}

impl MicrodeviceBaseModelController {
    pub async fn trigger_action<I, A>(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_id: String,
        microdevice_ids: I,
        action: A,
        payload: serde_json::Value,
    ) -> Result<Vec<MicrodeviceActionResponse>>
    where
        I: IntoIterator + Clone,
        I::IntoIter: ExactSizeIterator,
        I::Item: Into<MicrodeviceId>,
        A: Into<MicrodeviceAction> + Clone + Serialize,
    {
        let action: MicrodeviceAction = action.clone().into();

        // Fetch the microdevices
        let microdevice_data = Self::get_microdevice_from_cluster(
            mm,
            ctx,
            cluster_id,
            Some(microdevice_ids.clone()),
            None::<Vec<String>>,
            Some(true),
            None,
            Some(true),
        )
        .await?;

        // No microdevices found
        if microdevice_data.is_empty() {
            return Err(Error {
                kind: super::error::ErrorKind::MicrodeviceNotFound,
                message: "failed to execute action because microdevice was not found".to_string(),
            });
        }

        // Some microdevices were not found
        if microdevice_data.len() != microdevice_ids.into_iter().len() {
            return Err(Error {
                kind: super::error::ErrorKind::MicrodeviceNotFound,
                message: "failed to execute action because some microdevices were not found"
                    .to_string(),
            });
        }

        // Partition the microdevices into supported and not supported
        let (to_process, mut not_supported) =
            Self::partition_supported_microdevices(&microdevice_data, &action);

        // Collect the futures
        let fut: Vec<_> = to_process
            .into_iter()
            .map(|rec| Self::transmit_action(mm, rec.clone(), action.clone(), payload.clone()))
            .collect();

        // Execute the futures
        let action_reponses: Result<Vec<MicrodeviceActionResponse>> =
            futures::future::join_all(fut).await.into_iter().collect();

        // Combine the not supported and supported responses
        not_supported.extend(action_reponses?);

        Ok(not_supported)
    }

    fn partition_supported_microdevices(
        microdevices: &Vec<MicrodeviceRecord>,
        action: &MicrodeviceAction,
    ) -> (Vec<MicrodeviceRecord>, Vec<MicrodeviceActionResponse>) {
        let mut not_supported: Vec<MicrodeviceActionResponse> = vec![];
        let mut to_process: Vec<MicrodeviceRecord> = vec![];

        for microdevice in microdevices {
            if Self::is_action_supported(microdevice, action) {
                to_process.push(microdevice.clone());
            } else {
                not_supported.push(Self::unsupported_action_response(microdevice, action));
            }
        }

        (to_process, not_supported)
    }

    fn is_action_supported(rec: &MicrodeviceRecord, action: &MicrodeviceAction) -> bool {
        // Check if the user-defined action is supported by the microdevice
        if let MicrodeviceAction::UserDefined(action) = action {
            if let Some(topics) = &rec.topics {
                if let Some(topic_array) = topics.as_array() {
                    return topic_array
                        .iter()
                        .any(|v| v.as_str().unwrap_or_default().to_string() == *action);
                }
            }

            // If the microdevice has no topics or the topics are malformed
            return false;
        }

        // Execute the Default actions
        true
    }

    fn unsupported_action_response(
        microdevice: &MicrodeviceRecord,
        action: &MicrodeviceAction,
    ) -> MicrodeviceActionResponse {
        let message = match action {
            MicrodeviceAction::UserDefined(action_name) => {
                if microdevice.topics.is_none() {
                    format!(
                        "microdevice `{}` has no user-defined topics",
                        microdevice.name.clone().unwrap()
                    )
                } else {
                    format!(
                        "microdevice `{}` does not support action `{}`",
                        microdevice.name.clone().unwrap(),
                        action_name
                    )
                }
            },
            _ => panic!("It should not be possible to reach this point as the default actions are always supported."),
        };

        MicrodeviceActionResponse {
            microdevice_id: microdevice.id.unwrap().into(),
            status: "error".to_string(),
            message,
            payload: serde_json::Value::Null,
        }
    }

    async fn transmit_action(
        mm: &ModelManager,
        microdevice: MicrodeviceRecord,
        action: MicrodeviceAction,
        payload: serde_json::Value,
    ) -> Result<MicrodeviceActionResponse> {
        // Create the action message
        let action_message = MicrodeviceActionMessage {
            cluster_id: microdevice.cluster_id.unwrap().to_string(),
            microdevice_id: microdevice.id.unwrap().into(),
            action,
            payload,
        };

        // Serilize the action message
        let action_payload = serde_json::to_value(action_message)?;

        let res = mm.ampq_bridge.transmit_action(action_payload).await?;

        Ok(MicrodeviceActionResponse {
            microdevice_id: microdevice.id.unwrap().into(),
            status: "success".to_string(),
            message: "action was successfully transmitted".to_string(),
            payload: serde_json::to_string(&res).unwrap().into(),
        })
    }

    pub async fn get_microdevice_from_cluster<I, S>(
        mm: &ModelManager,
        ctx: &Ctx,
        cluster_uuid: String,
        microdevice_id: Option<I>,
        micodevice_name: Option<S>,
        inlcude_topics: Option<bool>,
        include_description: Option<bool>,
        include_cluster_id: Option<bool>,
    ) -> Result<Vec<MicrodeviceRecord>>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Into<MicrodeviceId>,
        S: IntoIterator,
        S::Item: Into<String>,
    {
        ClusterBMC::exists(mm, ctx, cluster_uuid.clone()).await?;

        let microdevice: Vec<MicrodeviceRecord> = microdevice::Entity::find()
            .select_only()
            .select_column(microdevice::Column::Id)
            .select_column(microdevice::Column::Name)
            .filter(microdevice::Column::ClusterId.eq(parse_cluster_id(&cluster_uuid)?))
            .apply_if(microdevice_id, |q, v| {
                q.filter(
                    microdevice::Column::Id.is_in(v.into_iter().map(|v| match v.into() {
                        MicrodeviceId::Id(id) => id,
                        MicrodeviceId::Name(_) => todo!(),
                    })),
                )
            })
            .apply_if(micodevice_name, |q, v| {
                q.filter(microdevice::Column::Name.is_in(v.into_iter().map(|v| v.into())))
            })
            .apply_if(inlcude_topics, |q, v| {
                if v {
                    q.select_column(microdevice::Column::Topics)
                } else {
                    q
                }
            })
            .apply_if(include_description, |q, v| {
                if v {
                    q.select_column(microdevice::Column::Description)
                } else {
                    q
                }
            })
            .apply_if(include_cluster_id, |q, v| {
                if v {
                    q.select_column(microdevice::Column::ClusterId)
                } else {
                    q
                }
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
    pub async fn delete_microdevice_from_cluster(
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
    pub async fn update_microdevice_in_cluster(
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

    pub async fn get_microdevice(
        ctx: &Ctx,
        mm: &ModelManager,
    ) -> Result<Option<MicrodeviceRecord>> {
        let microdevice_id = match ctx.get_microdevice_ids() {
            Some(v) => v,
            None => {
                return Err(Error {
                    kind: super::error::ErrorKind::InvalidContext,
                    message: "expected microdevice context".to_string(),
                })
            }
        };

        // convet microdevice id to integer
        let microdevice_id = match microdevice_id.0.parse::<i32>() {
            Ok(v) => v,
            Err(_) => {
                return Err(Error {
                    kind: super::error::ErrorKind::InvalidContext,
                    message: "invalid context".to_string(),
                })
            }
        };

        let rec: Option<MicrodeviceRecord> = microdevice::Entity::find_by_id(microdevice_id)
            .into_model()
            .one(&mm.db)
            .await?;

        Ok(rec)
    }
}
