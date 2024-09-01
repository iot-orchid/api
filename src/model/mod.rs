use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub(crate) db: DatabaseConnection,
}

// impl AppState {
//     pub fn new() -> Self {
//         Self {
//             db: DatabaseConnection::new(),
//         }
//     }
    
// }
