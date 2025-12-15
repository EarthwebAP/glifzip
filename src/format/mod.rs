pub mod header;
pub mod sidecar;

pub use header::{GlifHeader, MAGIC_NUMBER, GLIF_VERSION};
pub use sidecar::GlifSidecar;
