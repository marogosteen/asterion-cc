//! Type definitions for features.
//!
//! Provides type-safe representations of features.json content.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Feature lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
}

/// A single feature in the features.json file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Feature {
    /// Primary identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Alternative identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Alternative identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Human-readable description (also used as fallback identifier).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Test steps for Claude to execute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<String>>,

    /// Current status in the lifecycle.
    #[serde(default)]
    pub status: FeatureStatus,

    /// UUID assigned when feature transitions to in_progress.
    /// Only present for in_progress and completed features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iteration_id: Option<String>,

    /// Catch-all for unknown fields (forward compatibility).
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Feature {
    /// Get the display name using fallback chain: name -> id -> title -> description -> "(unnamed)".
    pub fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .or(self.id.as_deref())
            .or(self.title.as_deref())
            .or(self.description.as_deref())
            .unwrap_or("(unnamed)")
    }

    /// Check if feature is pending.
    pub fn is_pending(&self) -> bool {
        self.status == FeatureStatus::Pending
    }

    /// Check if feature is in progress.
    pub fn is_in_progress(&self) -> bool {
        self.status == FeatureStatus::InProgress
    }

    /// Check if feature is completed.
    pub fn is_completed(&self) -> bool {
        self.status == FeatureStatus::Completed
    }

    /// Start this feature (transitions to in_progress).
    pub fn start(&mut self, iteration_id: String) {
        self.status = FeatureStatus::InProgress;
        self.iteration_id = Some(iteration_id);
    }

    /// Complete this feature (transitions to completed).
    pub fn complete(&mut self) {
        self.status = FeatureStatus::Completed;
        // Keep iteration_id for historical tracking
    }
}

/// Root structure for features.json - supports both array and object formats.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FeaturesRoot {
    /// Array format: [ {...}, {...} ]
    Array(Vec<Feature>),

    /// Object format: { "features": [...], "project": "...", ... }
    Object {
        features: Vec<Feature>,
        #[serde(flatten)]
        metadata: HashMap<String, serde_json::Value>,
    },
}

impl FeaturesRoot {
    /// Get mutable access to features vec.
    pub fn features_mut(&mut self) -> &mut Vec<Feature> {
        match self {
            FeaturesRoot::Array(features) => features,
            FeaturesRoot::Object { features, .. } => features,
        }
    }

    /// Get immutable access to features vec.
    pub fn features(&self) -> &[Feature] {
        match self {
            FeaturesRoot::Array(features) => features,
            FeaturesRoot::Object { features, .. } => features,
        }
    }
}

/// Result of starting an iteration.
#[derive(Debug, Clone)]
pub struct IterationStart {
    /// Name/ID of the feature being worked on.
    pub feature_name: String,
    /// Unique identifier for this iteration (UUID v4).
    pub iteration_id: String,
    /// Index of the feature in the array.
    #[allow(dead_code)]
    pub feature_index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_display_name_priority() {
        let feature = Feature {
            name: Some("name".to_string()),
            id: Some("id".to_string()),
            title: Some("title".to_string()),
            description: Some("description".to_string()),
            steps: None,
            status: FeatureStatus::Pending,
            iteration_id: None,
            extra: HashMap::new(),
        };
        assert_eq!(feature.display_name(), "name");

        let feature = Feature {
            name: None,
            id: Some("id".to_string()),
            title: Some("title".to_string()),
            description: Some("description".to_string()),
            steps: None,
            status: FeatureStatus::Pending,
            iteration_id: None,
            extra: HashMap::new(),
        };
        assert_eq!(feature.display_name(), "id");
    }

    #[test]
    fn test_feature_status_transitions() {
        let mut feature = Feature {
            name: Some("test".to_string()),
            id: None,
            title: None,
            description: None,
            steps: None,
            status: FeatureStatus::Pending,
            iteration_id: None,
            extra: HashMap::new(),
        };

        assert!(feature.is_pending());
        assert!(!feature.is_in_progress());
        assert!(!feature.is_completed());

        feature.start("uuid-123".to_string());
        assert!(!feature.is_pending());
        assert!(feature.is_in_progress());
        assert!(!feature.is_completed());
        assert_eq!(feature.iteration_id, Some("uuid-123".to_string()));

        feature.complete();
        assert!(!feature.is_pending());
        assert!(!feature.is_in_progress());
        assert!(feature.is_completed());
        // iteration_id preserved
        assert_eq!(feature.iteration_id, Some("uuid-123".to_string()));
    }

    #[test]
    fn test_features_root_array_format() {
        let json = r#"[
            {"name": "feature1", "status": "pending"},
            {"name": "feature2", "status": "completed"}
        ]"#;
        let root: FeaturesRoot = serde_json::from_str(json).unwrap();
        assert_eq!(root.features().len(), 2);
        assert_eq!(root.features()[0].display_name(), "feature1");
    }

    #[test]
    fn test_features_root_object_format() {
        let json = r#"{
            "project": "my-project",
            "features": [
                {"name": "feature1", "status": "pending"}
            ]
        }"#;
        let root: FeaturesRoot = serde_json::from_str(json).unwrap();
        assert_eq!(root.features().len(), 1);
        assert_eq!(root.features()[0].display_name(), "feature1");

        // Check that metadata is preserved
        if let FeaturesRoot::Object { metadata, .. } = &root {
            assert!(metadata.contains_key("project"));
        } else {
            panic!("Expected Object variant");
        }
    }

    #[test]
    fn test_feature_status_default() {
        let json = r#"{"name": "feature1"}"#;
        let feature: Feature = serde_json::from_str(json).unwrap();
        assert_eq!(feature.status, FeatureStatus::Pending);
    }

    #[test]
    fn test_feature_extra_fields_preserved() {
        let json = r#"{"name": "feature1", "status": "pending", "custom_field": "custom_value"}"#;
        let feature: Feature = serde_json::from_str(json).unwrap();
        assert!(feature.extra.contains_key("custom_field"));

        // Serialize and check round-trip
        let serialized = serde_json::to_string(&feature).unwrap();
        assert!(serialized.contains("custom_field"));
    }
}
