//! <https://datatracker.ietf.org/doc/html/draft-sharabayko-srt>

pub mod connection;
pub mod macros;
pub mod protocol;
pub mod server;

pub use server::{callback_connection::CallbackConnection, callback_server::CallbackServer};
