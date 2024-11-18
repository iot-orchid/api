#[derive(Clone, Debug)]
pub enum Ctx {
    UserCtx {
        user_id: String,
    },
    MicrodeviceCtx {
        device_id: String,
        cluster_id: String,
    },
}

impl Ctx {
    pub fn new_user<S>(uuid: S) -> Ctx
    where
        S: Into<String>,
    {
        Self::UserCtx {
            user_id: uuid.into(),
        }
    }

    pub fn get_user_id(&self) -> Option<&String> {
        if let Ctx::UserCtx { user_id, .. } = self {
            Some(user_id)
        } else {
            None
        }
    }

    pub fn get_microdevice_ids(&self) -> Option<(&String, &String)> {
        if let Ctx::MicrodeviceCtx {
            device_id,
            cluster_id,
        } = self
        {
            Some((device_id, cluster_id))
        } else {
            None
        }
    }
}
