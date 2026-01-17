//! Features module for managing feature lifecycle.
//!
//! This module provides type-safe representations and operations for
//! features.json files.

mod repository;
mod types;

pub use repository::FeatureRepository;

// Re-export types for external use if needed
#[allow(unused_imports)]
pub use types::{Feature, FeatureStatus, FeaturesRoot, IterationStart};
