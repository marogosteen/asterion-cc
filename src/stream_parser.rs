//! Stream parser for Claude Code's `--output-format stream-json` NDJSON output.
//!
//! Parses streaming events from Claude Code to extract tool usage and text content
//! for progress display.

use serde_json::Value;

/// Represents a parsed event from Claude Code's stream-json output.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    /// Tool usage started (e.g., Read, Write, Bash)
    ToolUse {
        tool_name: String,
    },

    /// Text content from Claude (assistant message)
    Text {
        content: String,
    },

    /// Result event signaling completion
    Result {
        success: bool,
    },

    /// Unknown or unparseable event (ignored for forward compatibility)
    Unknown,
}


/// Parse a single NDJSON line into a StreamEvent.
///
/// Returns `StreamEvent::Unknown` for unparseable or unrecognized events.
/// This ensures forward compatibility with future Claude Code versions.
pub fn parse_line(line: &str) -> StreamEvent {
    let line = line.trim();
    if line.is_empty() {
        return StreamEvent::Unknown;
    }

    // Try to parse as JSON
    let value: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(_) => return StreamEvent::Unknown,
    };

    // Extract event type
    let event_type = match value.get("type").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return StreamEvent::Unknown,
    };

    match event_type {
        // Tool use events from content_block_start
        "content_block_start" => {
            if let Some(block) = value.get("content_block") {
                let block_type = block.get("type").and_then(|v| v.as_str());
                if block_type == Some("tool_use") {
                    if let Some(name) = block.get("name").and_then(|v| v.as_str()) {
                        return StreamEvent::ToolUse {
                            tool_name: name.to_string(),
                        };
                    }
                } else if block_type == Some("text") {
                    // Text block starting - we'll get the content in deltas
                    return StreamEvent::Unknown;
                }
            }
            StreamEvent::Unknown
        }

        // Text delta events
        "content_block_delta" => {
            if let Some(delta) = value.get("delta") {
                let delta_type = delta.get("type").and_then(|v| v.as_str());
                if delta_type == Some("text_delta") {
                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                        return StreamEvent::Text {
                            content: text.to_string(),
                        };
                    }
                }
            }
            StreamEvent::Unknown
        }

        // Assistant message with tool_use in content
        "assistant" => {
            if let Some(message) = value.get("message") {
                if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
                    for item in content {
                        let item_type = item.get("type").and_then(|v| v.as_str());
                        if item_type == Some("tool_use") {
                            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                                return StreamEvent::ToolUse {
                                    tool_name: name.to_string(),
                                };
                            }
                        }
                    }
                }
            }
            StreamEvent::Unknown
        }

        // Result event
        "result" => {
            let subtype = value.get("subtype").and_then(|v| v.as_str());
            StreamEvent::Result {
                success: subtype == Some("success"),
            }
        }

        // All other events (message_start, message_stop, etc.)
        _ => StreamEvent::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_use_content_block() {
        let json = r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_123","name":"Read"}}"#;
        let event = parse_line(json);
        assert_eq!(
            event,
            StreamEvent::ToolUse {
                tool_name: "Read".to_string()
            }
        );
    }

    #[test]
    fn test_parse_text_delta() {
        let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
        let event = parse_line(json);
        assert_eq!(
            event,
            StreamEvent::Text {
                content: "Hello".to_string()
            }
        );
    }

    #[test]
    fn test_parse_assistant_tool_use() {
        let json = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"123","name":"Bash"}]}}"#;
        let event = parse_line(json);
        assert_eq!(
            event,
            StreamEvent::ToolUse {
                tool_name: "Bash".to_string()
            }
        );
    }

    #[test]
    fn test_parse_result_success() {
        let json = r#"{"type":"result","subtype":"success","result":"done"}"#;
        let event = parse_line(json);
        assert_eq!(event, StreamEvent::Result { success: true });
    }

    #[test]
    fn test_parse_result_error() {
        let json = r#"{"type":"result","subtype":"error","error":"something went wrong"}"#;
        let event = parse_line(json);
        assert_eq!(event, StreamEvent::Result { success: false });
    }

    #[test]
    fn test_parse_unknown_event() {
        let json = r#"{"type":"message_start","message":{}}"#;
        let event = parse_line(json);
        assert_eq!(event, StreamEvent::Unknown);
    }

    #[test]
    fn test_parse_empty_line() {
        assert_eq!(parse_line(""), StreamEvent::Unknown);
        assert_eq!(parse_line("  "), StreamEvent::Unknown);
    }

    #[test]
    fn test_parse_invalid_json() {
        assert_eq!(parse_line("not json"), StreamEvent::Unknown);
        assert_eq!(parse_line("{broken"), StreamEvent::Unknown);
    }
}
