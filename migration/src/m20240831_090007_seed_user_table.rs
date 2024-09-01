use entity::cluster;
#[allow(unused_imports)]
use entity::microdevice;
use entity::user;
use entity::user_cluster;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::entity::*;
use uuid;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let user_uuid = uuid::Uuid::new_v4();

        // Control user with no clusters or microdevices
        let _res = user::ActiveModel {
            id: Set(user_uuid),
            username: Set("control".to_string()),
            password_hash: Set(bcrypt::hash("control", bcrypt::DEFAULT_COST).unwrap()),
        }
        .insert(db)
        .await
        .unwrap();

        let user_uuid = uuid::Uuid::new_v4();

        let res = user::ActiveModel {
            id: Set(user_uuid),
            username: Set("foo".to_string()),
            password_hash: Set(bcrypt::hash("bar", bcrypt::DEFAULT_COST).unwrap()),
        }
        .insert(db)
        .await;
        match res {
            Ok(_) => (),
            Err(e) => return Err(DbErr::from(e)),
        }

        for i in 0..10 {
            let cluster_uuid = uuid::Uuid::new_v4();
            let res = cluster::ActiveModel {
                id: Set(cluster_uuid),
                name: Set(format!("factory-{}", i)),
                ..Default::default()
            }
            .insert(db)
            .await;
            match res {
                Ok(_) => (),
                Err(e) => return Err(DbErr::from(e)),
            }

            let res = user_cluster::ActiveModel {
                user_id: Set(user_uuid),
                cluster_id: Set(cluster_uuid),
            }
            .insert(db)
            .await;

            match res {
                Ok(_) => (),
                Err(e) => return Err(DbErr::from(e)),
            }

            for j in 0..5 {
                let res = microdevice::ActiveModel {
                    cluster_id: Set(cluster_uuid),
                    name: Set(format!("microdevice-{}", j)),
                    description: Set("sensor".to_string()),
                    ..Default::default()
                }
                .insert(db)
                .await;

                match res {
                    Ok(_) => (),
                    Err(e) => return Err(DbErr::from(e)),
                }
            }
        }

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
