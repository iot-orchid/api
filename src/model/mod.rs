use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct ModelManager {
    pub(crate) db: DatabaseConnection,
}

// impl AppState {
//     pub fn new() -> Self {
//         Self {
//             db: DatabaseConnection::new(),
//         }
//     }

// }
