//! Shared request/response models and error envelope.
//!
//! This module is the Rust source of truth for all IPC contracts.
//! Frontend TypeScript mirrors live in `src/lib/types/`.

pub mod error;
pub mod files;
pub mod git;
pub mod links;
pub mod search;
pub mod status;
pub mod workspace;

// Re-export the error type at the models root for ergonomic imports:
//   use crate::models::VigilError;
pub use error::VigilError;
