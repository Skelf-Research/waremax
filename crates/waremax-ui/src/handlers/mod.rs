//! HTTP request handlers for the web UI

pub mod api;
pub mod websocket;
pub mod static_files;

pub use api::*;
pub use websocket::*;
pub use static_files::*;
