#[cfg(feature = "api")]
pub mod routes;
#[cfg(feature = "api")]
pub mod server;

#[cfg(feature = "api")]
pub use server::serve;
