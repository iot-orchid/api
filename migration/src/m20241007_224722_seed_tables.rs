use entity::{cluster, microdevice, user, user_cluster};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::Set;
#[allow(unused_imports)]
use sea_orm_migration::{prelude::*, schema::*};
use std::collections::HashMap;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();

        let users: [user::ActiveModel; 3] = [
            user::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                username: Set("foo".to_string()),
                password_hash: Set(bcrypt::hash("bar", bcrypt::DEFAULT_COST).unwrap()),
            },
            user::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                username: Set("baz".to_string()),
                password_hash: Set(bcrypt::hash("qux", bcrypt::DEFAULT_COST).unwrap()),
            },
            user::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                username: Set("control".to_string()),
                password_hash: Set(bcrypt::hash("control", bcrypt::DEFAULT_COST).unwrap()),
            },
        ];

        let map: HashMap<
            Uuid,
            (
                Vec<cluster::ActiveModel>,
                Vec<Vec<microdevice::ActiveModel>>,
            ),
        > = users.iter().fold(HashMap::new(), |mut acc, u| {
            let clusters: Vec<cluster::ActiveModel> = (0..3)
                .map(|i| cluster::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    name: Set(format!("{}-cluster-{}", u.username.clone().unwrap(), i)),
                    ..Default::default()
                })
                .collect();
            let mut md_count = -1;
            let microdevices: Vec<Vec<microdevice::ActiveModel>> = clusters
                .iter()
                .map(|c| {
                    (0..3)
                        .map(|i| {
                            md_count += 1;
                            microdevice::ActiveModel {
                                name: Set(format!("{}-microdevice-{}", c.name.clone().unwrap(), i)),
                                description: Set(Some(format!(
                                    "I belong to {} and am part of the cluster {} my name is {}",
                                    u.username.clone().unwrap(),
                                    c.name.clone().unwrap(),
                                    i
                                ))),
                                cluster_id: Set(c.id.clone().unwrap()),
                                ..Default::default()
                            }
                        })
                        .collect()
                })
                .collect();

            acc.insert(u.id.clone().unwrap(), (clusters, microdevices));

            acc
        });

        let fut: Vec<_> = users.into_iter().map(|u| u.insert(db)).collect();
        futures::future::try_join_all(fut).await?;

        let (cluster_fut, user_cluster_fut, microdevices_fut): (Vec<_>, Vec<_>, Vec<Vec<_>>) = map
            .into_iter()
            .map(|(user_uuid, (clusters, microdevices))| {
                let cluster_fut: Vec<_> =
                    clusters.clone().into_iter().map(|c| c.insert(db)).collect();

                let user_cluster_fut: Vec<_> = clusters
                    .clone()
                    .into_iter()
                    .map(|c| {
                        user_cluster::ActiveModel {
                            user_id: Set(user_uuid.clone()),
                            cluster_id: Set(c.id.clone().unwrap()),
                            ..Default::default()
                        }
                        .insert(db)
                    })
                    .collect();

                let microdevices_fut: Vec<Vec<_>> = microdevices
                    .into_iter()
                    .map(|m| m.into_iter().map(|m| m.insert(db)).collect())
                    .collect();

                (cluster_fut, user_cluster_fut, microdevices_fut)
            })
            .fold(
                (Vec::new(), Vec::new(), Vec::new()),
                |mut acc, (cf, ucf, mf)| {
                    acc.0.extend(cf);
                    acc.1.extend(ucf);
                    acc.2.extend(mf);
                    acc
                },
            );

        futures::future::try_join_all(cluster_fut).await?;
        futures::future::try_join_all(user_cluster_fut).await?;
        futures::future::try_join_all(microdevices_fut.into_iter().flatten().collect::<Vec<_>>())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let users = user::Entity::find().all(db).await?;

        let fut: Vec<_> = users.into_iter().map(|u| u.delete(db)).collect();

        futures::future::try_join_all(fut).await?;

        let clusters = cluster::Entity::find().all(db).await?;

        let fut: Vec<_> = clusters.into_iter().map(|c| c.delete(db)).collect();

        futures::future::try_join_all(fut).await?;

        let user_clusters = user_cluster::Entity::find().all(db).await?;

        let fut: Vec<_> = user_clusters.into_iter().map(|uc| uc.delete(db)).collect();

        futures::future::try_join_all(fut).await?;

        let microdevices = microdevice::Entity::find().all(db).await?;

        let fut: Vec<_> = microdevices.into_iter().map(|m| m.delete(db)).collect();

        futures::future::try_join_all(fut).await?;

        Ok(())
    }
}
