//! Browser package shim for Vicia DB.
//!
//! This is the durable package boundary used by local Vetch development and,
//! later, the public `@vicia-db/browser` release. The core crate still carries
//! the core `vicia-db` crate; consumers only see the Vicia surface.

pub use vicia_db::*;
