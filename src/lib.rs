mod config;
mod routes;
mod services;
mod storage;

pub use routes::*;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::routes::*;
    pub use crate::services::*;
    pub use crate::storage::*;
}
