use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use uuid::Uuid;

/// Status values for features
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_IN_PROGRESS: &str = "in_progress";
pub const STATUS_COMPLETED: &str = "completed";

/// Result of starting an iteration
pub struct IterationStart {
    /// Name/ID of the feature being worked on
    pub feature_name: String,
    /// Unique identifier for this iteration (UUID v4)
    pub iteration_id: String,
    /// Index of the feature in the array (for future use)
    #[allow(dead_code)]
    pub feature_index: usize,
}

/// Read the features array from the features file
fn read_features(path: &Path) -> Result<(Value, Vec<Value>)> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;

    let root: Value = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    let features = match &root {
        Value::Array(arr) => arr.clone(),
        Value::Object(obj) => obj
            .get("features")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => bail!("features file must be an array or object with 'features' key"),
    };

    Ok((root, features))
}

/// Write the features back to the features file
fn write_features(path: &Path, root: &Value, features: Vec<Value>) -> Result<()> {
    let updated = match root {
        Value::Array(_) => Value::Array(features),
        Value::Object(obj) => {
            let mut obj = obj.clone();
            obj.insert("features".to_string(), Value::Array(features));
            Value::Object(obj)
        }
        _ => bail!("unexpected root type"),
    };

    let content = serde_json::to_string_pretty(&updated)?;
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

/// Get the status of a feature
fn get_status(feature: &Value) -> Option<&str> {
    feature.get("status").and_then(|v| v.as_str())
}

/// Get the name of a feature (tries "name", "id", "title", "description" fields)
fn get_feature_name(feature: &Value) -> String {
    feature
        .get("name")
        .or_else(|| feature.get("id"))
        .or_else(|| feature.get("title"))
        .or_else(|| feature.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("(unnamed)")
        .to_string()
}

/// Check if all features in the features file are completed
pub fn all_completed(path: &Path) -> Result<bool> {
    let (_, features) = read_features(path)?;

    Ok(features
        .iter()
        .all(|f| get_status(f) == Some(STATUS_COMPLETED)))
}

/// Start an iteration: find the first pending feature and mark it as in_progress
///
/// Returns the feature name/index, or None if no pending features
pub fn start_iteration(path: &Path) -> Result<Option<IterationStart>> {
    let (root, mut features) = read_features(path)?;

    // Find the first pending feature
    let pending_index = features
        .iter()
        .position(|f| get_status(f) == Some(STATUS_PENDING));

    let Some(index) = pending_index else {
        return Ok(None);
    };

    // Get feature name before modifying
    let feature_name = get_feature_name(&features[index]);

    // Generate unique iteration ID
    let iteration_id = Uuid::new_v4().to_string();

    // Update status to in_progress and add iteration_id
    if let Some(obj) = features[index].as_object_mut() {
        obj.insert(
            "status".to_string(),
            Value::String(STATUS_IN_PROGRESS.to_string()),
        );
        obj.insert(
            "iteration_id".to_string(),
            Value::String(iteration_id.clone()),
        );
    }

    write_features(path, &root, features)?;

    Ok(Some(IterationStart {
        feature_name,
        iteration_id,
        feature_index: index,
    }))
}

/// Complete an iteration: mark the in_progress feature as completed
///
/// Returns the feature name, or None if no in_progress feature found
pub fn complete_iteration(path: &Path) -> Result<Option<String>> {
    let (root, mut features) = read_features(path)?;

    // Find the in_progress feature
    let in_progress_index = features
        .iter()
        .position(|f| get_status(f) == Some(STATUS_IN_PROGRESS));

    let Some(index) = in_progress_index else {
        return Ok(None);
    };

    // Get feature name before modifying
    let feature_name = get_feature_name(&features[index]);

    // Update status to completed
    if let Some(obj) = features[index].as_object_mut() {
        obj.insert(
            "status".to_string(),
            Value::String(STATUS_COMPLETED.to_string()),
        );
    }

    write_features(path, &root, features)?;

    Ok(Some(feature_name))
}

/// Get the current in_progress feature name, if any
pub fn get_current_feature(path: &Path) -> Result<Option<String>> {
    let (_, features) = read_features(path)?;

    let in_progress = features
        .iter()
        .find(|f| get_status(f) == Some(STATUS_IN_PROGRESS));

    Ok(in_progress.map(get_feature_name))
}

/// Peek at the next pending feature without modifying the file
pub fn peek_next_pending(path: &Path) -> Result<Option<String>> {
    let (_, features) = read_features(path)?;

    let pending = features
        .iter()
        .find(|f| get_status(f) == Some(STATUS_PENDING));

    Ok(pending.map(get_feature_name))
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
        assert!(all_completed(file.path()).unwrap());
    }

    #[test]
    fn test_not_all_completed_with_array_format() {
        let content = r#"[
            {"name": "feature1", "status": "completed"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);
        assert!(!all_completed(file.path()).unwrap());
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
        assert!(all_completed(file.path()).unwrap());
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
        assert!(!all_completed(file.path()).unwrap());
    }

    #[test]
    fn test_empty_features() {
        let content = r#"[]"#;
        let file = write_temp_file(content);
        assert!(all_completed(file.path()).unwrap());
    }

    #[test]
    fn test_missing_status() {
        let content = r#"[{"name": "feature1"}]"#;
        let file = write_temp_file(content);
        assert!(!all_completed(file.path()).unwrap());
    }

    #[test]
    fn test_start_iteration_marks_pending_as_in_progress() {
        let content = r#"[
            {"name": "feature1", "status": "pending"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);

        let result = start_iteration(file.path()).unwrap().unwrap();
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

        let result = start_iteration(file.path()).unwrap().unwrap();
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

        let result = start_iteration(file.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_complete_iteration_marks_in_progress_as_completed() {
        let content = r#"[
            {"name": "feature1", "status": "in_progress"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);

        let result = complete_iteration(file.path()).unwrap().unwrap();
        assert_eq!(result, "feature1");

        // Verify file was updated
        let updated = read_temp_file(&file);
        let parsed: Value = serde_json::from_str(&updated).unwrap();
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

        let result = complete_iteration(file.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_full_iteration_cycle() {
        let content = r#"[
            {"name": "feature1", "status": "pending"},
            {"name": "feature2", "status": "pending"}
        ]"#;
        let file = write_temp_file(content);

        // Start iteration 1
        let start1 = start_iteration(file.path()).unwrap().unwrap();
        assert_eq!(start1.feature_name, "feature1");

        // Complete iteration 1
        let complete1 = complete_iteration(file.path()).unwrap().unwrap();
        assert_eq!(complete1, "feature1");

        // Start iteration 2
        let start2 = start_iteration(file.path()).unwrap().unwrap();
        assert_eq!(start2.feature_name, "feature2");

        // Complete iteration 2
        let complete2 = complete_iteration(file.path()).unwrap().unwrap();
        assert_eq!(complete2, "feature2");

        // All completed
        assert!(all_completed(file.path()).unwrap());
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

        // Start iteration
        let start = start_iteration(file.path()).unwrap().unwrap();
        assert_eq!(start.feature_name, "feature1");

        // Verify other fields are preserved
        let updated: Value = serde_json::from_str(&read_temp_file(&file)).unwrap();
        assert_eq!(updated["name"], "my-project");
        assert_eq!(updated["features"][0]["status"], "in_progress");
    }
}
