#[cfg(feature = "external-sources")]
pub mod drand;

#[cfg(feature = "external-sources")]
pub use drand::*;
