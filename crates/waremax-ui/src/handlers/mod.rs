//! HTTP request handlers for the web UI

pub mod api;
pub mod static_files;
pub mod websocket;

pub use api::*;
pub use static_files::*;
pub use websocket::*;
