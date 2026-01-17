//! Prompt building for the Claude agent.
//!
//! Contains templates and logic for constructing prompts sent to Claude Code.

/// Build the prompt for the agent.
///
/// The feature_name is the specific feature to implement in this iteration.
/// The iteration_id uniquely identifies this iteration for feature lookup.
pub fn build_prompt(
    features_ref: &str,
    progress_ref: &str,
    feature_name: &str,
    iteration_id: &str,
) -> String {
    format!(
        r#"Read the following files:
- {features_ref}
- {progress_ref}

You are an autonomous software engineer. Your task is to implement a single feature.

## Current Task

Implement the feature: **{feature_name}**
Iteration ID: `{iteration_id}`

Find this feature in {features_ref} by matching the `iteration_id` field.

## Workflow

1. Read the features file and locate the feature with `iteration_id: "{iteration_id}"`.
2. Run `./.asterion/init.sh` to verify the project is in a clean state before changes.
3. Implement ONLY the specified feature following any project-specific instructions.
4. Run `./.asterion/init.sh` again to verify build, tests, and lint pass.
5. Append a concise progress note to {progress_ref}.
6. If this is a git repository, commit your changes.
7. Exit immediately after committing.

## Constraints

- Implement ONLY the feature with iteration_id "{iteration_id}". Do not work on other features.
- Stop after completing this single feature. Do not proceed to the next feature.
- Do NOT modify {features_ref} (status is managed externally).
- Follow the project structure defined in the features file.
- Do not modify unrelated code.
- Do not print, log, or commit secrets or credentials.
- Do not run destructive commands."#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_contains_feature_name() {
        let prompt = build_prompt(
            ".asterion/features.json",
            ".asterion/progress.md",
            "test-feature",
            "abc-123",
        );
        assert!(prompt.contains("test-feature"));
        assert!(prompt.contains("abc-123"));
    }

    #[test]
    fn test_build_prompt_contains_file_references() {
        let prompt = build_prompt(
            ".asterion/features.json",
            ".asterion/progress.md",
            "test-feature",
            "abc-123",
        );
        assert!(prompt.contains(".asterion/features.json"));
        assert!(prompt.contains(".asterion/progress.md"));
    }

    #[test]
    fn test_build_prompt_contains_constraints() {
        let prompt = build_prompt(
            ".asterion/features.json",
            ".asterion/progress.md",
            "test-feature",
            "abc-123",
        );
        assert!(prompt.contains("Do NOT modify"));
        assert!(prompt.contains("Do not run destructive commands"));
    }
}
