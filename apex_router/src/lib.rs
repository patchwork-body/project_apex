#![allow(missing_docs)]

mod client_router;
mod get_matched_path;
pub mod init_data;
mod server_router;

pub use client_router::{ApexClientRoute, ApexClientRouter};
pub(crate) use get_matched_path::get_matched_path;
pub use server_router::{ApexServerHandler, ApexServerRoute, ApexServerRouter};
