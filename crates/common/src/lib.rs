mod domain;
mod events;
pub mod kafka;

pub use domain::*;
pub use events::*;

pub mod proto {
    tonic::include_proto!("media.v1");
}
