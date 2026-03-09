//! <https://datatracker.ietf.org/doc/html/draft-sharabayko-srt>

pub mod macros;
pub mod protocol;
pub mod server;

pub use server::callback::{connection::CallbackConnection, listener::CallbackListener};
#[cfg(feature = "tokio")]
pub use server::tokio::{connection::AsyncConnection, listener::AsyncListener};
