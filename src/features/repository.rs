//! Feature repository for file I/O operations.
//!
//! Provides a repository pattern abstraction over features.json file operations.

use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::types::{FeaturesRoot, IterationStart};
use crate::error::{FeaturesError, FeaturesResult};

/// Repository for managing features in the features.json file.
#[derive(Debug, Clone)]
pub struct FeatureRepository {
    path: PathBuf,
}

impl FeatureRepository {
    /// Create a new repository for the given file path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Get the path to the features file.
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read and parse the features file.
    fn read(&self) -> FeaturesResult<FeaturesRoot> {
        let content = fs::read_to_string(&self.path).map_err(|source| FeaturesError::ReadError {
            path: self.path.clone(),
            source,
        })?;

        let root: FeaturesRoot =
            serde_json::from_str(&content).map_err(|source| FeaturesError::ParseError {
                path: self.path.clone(),
                source,
            })?;

        Ok(root)
    }

    /// Write the features back to the file.
    fn write(&self, root: &FeaturesRoot) -> FeaturesResult<()> {
        let content = serde_json::to_string_pretty(root).map_err(|source| {
            FeaturesError::ParseError {
                path: self.path.clone(),
                source,
            }
        })?;

        fs::write(&self.path, content).map_err(|source| FeaturesError::WriteError {
            path: self.path.clone(),
            source,
        })?;

        Ok(())
    }

    /// Check if all features are completed.
    pub fn all_completed(&self) -> FeaturesResult<bool> {
        let root = self.read()?;
        Ok(root.features().iter().all(|f| f.is_completed()))
    }

    /// Start an iteration: find the first pending feature and mark it as in_progress.
    ///
    /// Returns the feature info, or None if no pending features.
    pub fn start_iteration(&self) -> FeaturesResult<Option<IterationStart>> {
        let mut root = self.read()?;
        let features = root.features_mut();

        // Find the first pending feature
        let pending_idx = features.iter().position(|f| f.is_pending());

        let Some(idx) = pending_idx else {
            return Ok(None);
        };

        let feature = &mut features[idx];
        let feature_name = feature.display_name().to_string();
        let iteration_id = Uuid::new_v4().to_string();

        feature.start(iteration_id.clone());

        self.write(&root)?;

        Ok(Some(IterationStart {
            feature_name,
            iteration_id,
            feature_index: idx,
        }))
    }

    /// Complete an iteration: mark the in_progress feature as completed.
    ///
    /// Returns the feature name, or None if no in_progress feature found.
    pub fn complete_iteration(&self) -> FeaturesResult<Option<String>> {
        let mut root = self.read()?;
        let features = root.features_mut();

        // Find the in_progress feature
        let in_progress_idx = features.iter().position(|f| f.is_in_progress());

        let Some(idx) = in_progress_idx else {
            return Ok(None);
        };

        let feature = &mut features[idx];
        let feature_name = feature.display_name().to_string();

        feature.complete();

        self.write(&root)?;

        Ok(Some(feature_name))
    }

    /// Get the current in_progress feature name, if any.
    pub fn get_current_feature(&self) -> FeaturesResult<Option<String>> {
        let root = self.read()?;

        let in_progress = root.features().iter().find(|f| f.is_in_progress());

        Ok(in_progress.map(|f| f.display_name().to_string()))
    }

    /// Peek at the next pending feature without modifying the file.
    pub fn peek_next_pending(&self) -> FeaturesResult<Option<String>> {
        let root = self.read()?;

        let pending = root.features().iter().find(|f| f.is_pending());

        Ok(pending.map(|f| f.display_name().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    fn read_temp_file(file: &NamedTempFile) -> String {
        fs::read_to_string(file.path()).unwrap()
    }

    #[test]
    fn test_all_completed_with_array_format() {
        let content = r#"[
            {"name": "feature1", "status": "completed"},
            {"name": "feature2", "status": "completed"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());
        assert!(repo.all_completed().unwrap());
    }

    #[test]
    fn test_not_all_completed_with_array_format() {
        let content = r#"[
            {"name": "feature1", "status": "completed"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());
        assert!(!repo.all_completed().unwrap());
    }

    #[test]
    fn test_all_completed_with_object_format() {
        let content = r#"{
            "features": [
                {"name": "feature1", "status": "completed"},
                {"name": "feature2", "status": "completed"}
            ]
        }"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());
        assert!(repo.all_completed().unwrap());
    }

    #[test]
    fn test_not_all_completed_with_object_format() {
        let content = r#"{
            "features": [
                {"name": "feature1", "status": "completed"},
                {"name": "feature2", "status": "in_progress"}
            ]
        }"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());
        assert!(!repo.all_completed().unwrap());
    }

    #[test]
    fn test_empty_features() {
        let content = r#"[]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());
        assert!(repo.all_completed().unwrap());
    }

    #[test]
    fn test_missing_status() {
        let content = r#"[{"name": "feature1"}]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());
        assert!(!repo.all_completed().unwrap());
    }

    #[test]
    fn test_start_iteration_marks_pending_as_in_progress() {
        let content = r#"[
            {"name": "feature1", "status": "pending"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        let result = repo.start_iteration().unwrap().unwrap();
        assert_eq!(result.feature_name, "feature1");
        assert_eq!(result.feature_index, 0);
        // iteration_id should be a valid UUID (36 chars with hyphens)
        assert_eq!(result.iteration_id.len(), 36);
        assert!(result.iteration_id.contains('-'));

        // Verify file was updated with status and iteration_id
        let updated = read_temp_file(&file);
        assert!(updated.contains(r#""status": "in_progress""#));
        assert!(updated.contains(&format!(r#""iteration_id": "{}""#, result.iteration_id)));
    }

    #[test]
    fn test_start_iteration_skips_completed() {
        let content = r#"[
            {"name": "feature1", "status": "completed"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        let result = repo.start_iteration().unwrap().unwrap();
        assert_eq!(result.feature_name, "feature2");
        assert_eq!(result.feature_index, 1);
    }

    #[test]
    fn test_start_iteration_returns_none_when_all_completed() {
        let content = r#"[
            {"name": "feature1", "status": "completed"},
            {"name": "feature2", "status": "completed"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        let result = repo.start_iteration().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_complete_iteration_marks_in_progress_as_completed() {
        let content = r#"[
            {"name": "feature1", "status": "in_progress"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        let result = repo.complete_iteration().unwrap().unwrap();
        assert_eq!(result, "feature1");

        // Verify file was updated
        let updated = read_temp_file(&file);
        let parsed: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(parsed[0]["status"], "completed");
        assert_eq!(parsed[1]["status"], "pending");
    }

    #[test]
    fn test_complete_iteration_returns_none_when_no_in_progress() {
        let content = r#"[
            {"name": "feature1", "status": "completed"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        let result = repo.complete_iteration().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_full_iteration_cycle() {
        let content = r#"[
            {"name": "feature1", "status": "pending"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        // Start iteration 1
        let start1 = repo.start_iteration().unwrap().unwrap();
        assert_eq!(start1.feature_name, "feature1");

        // Complete iteration 1
        let complete1 = repo.complete_iteration().unwrap().unwrap();
        assert_eq!(complete1, "feature1");

        // Start iteration 2
        let start2 = repo.start_iteration().unwrap().unwrap();
        assert_eq!(start2.feature_name, "feature2");

        // Complete iteration 2
        let complete2 = repo.complete_iteration().unwrap().unwrap();
        assert_eq!(complete2, "feature2");

        // All completed
        assert!(repo.all_completed().unwrap());
    }

    #[test]
    fn test_object_format_iteration() {
        let content = r#"{
            "name": "my-project",
            "features": [
                {"name": "feature1", "status": "pending"}
            ]
        }"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        // Start iteration
        let start = repo.start_iteration().unwrap().unwrap();
        assert_eq!(start.feature_name, "feature1");

        // Verify other fields are preserved
        let updated: serde_json::Value = serde_json::from_str(&read_temp_file(&file)).unwrap();
        assert_eq!(updated["name"], "my-project");
        assert_eq!(updated["features"][0]["status"], "in_progress");
    }

    #[test]
    fn test_get_current_feature() {
        let content = r#"[
            {"name": "feature1", "status": "in_progress"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        let current = repo.get_current_feature().unwrap();
        assert_eq!(current, Some("feature1".to_string()));
    }

    #[test]
    fn test_peek_next_pending() {
        let content = r#"[
            {"name": "feature1", "status": "completed"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        let repo = FeatureRepository::new(file.path());

        let next = repo.peek_next_pending().unwrap();
        assert_eq!(next, Some("feature2".to_string()));
    }
}
