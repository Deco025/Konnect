//! Shared, compatibility-preserving tool outcome metadata.
//!
//! Tool-specific response fields remain authoritative for their domain. This
//! envelope gives callers and observability one common answer to whether the
//! requested work completed, partially completed, failed before mutation, or
//! may have changed state before verification failed.

use crate::mcp::protocol::{CallToolResult, ToolContent};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeStatus {
    Complete,
    Partial,
    Failed,
    Uncertain,
}

impl OutcomeStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Uncertain => "uncertain",
        }
    }
}

pub fn summary(
    status: OutcomeStatus,
    target: impl Into<String>,
    source: impl Into<String>,
    requested: usize,
    completed: usize,
    failed: usize,
    retry: Option<Value>,
) -> Value {
    debug_assert_eq!(
        requested,
        completed.saturating_add(failed),
        "outcome counts must account for every requested item or check"
    );
    json!({
        "status": status,
        "target": target.into(),
        "source": source.into(),
        "requested": requested,
        "completed": completed,
        "failed": failed,
        "retry": retry
    })
}

pub fn retry_failed_items(indexes: Vec<usize>) -> Value {
    json!({
        "safe": true,
        "scope": "failed_items",
        "item_indexes": indexes,
        "instruction": "correct and retry only the listed input items"
    })
}

pub fn retry_whole_request() -> Value {
    json!({
        "safe": true,
        "scope": "whole_request",
        "instruction": "correct the refusal and retry the request; no item was applied"
    })
}

pub fn inspect_before_retry() -> Value {
    json!({
        "safe": false,
        "scope": "inspect_target",
        "instruction": "reload and inspect the target before retrying; do not blindly repeat the mutation"
    })
}

/// Add the shared envelope while preserving every existing response field.
pub fn attach(mut result: CallToolResult, outcome: Value) -> CallToolResult {
    let Some(ToolContent::Text { text }) = result.content.first_mut() else {
        return result;
    };
    let Ok(mut body) = serde_json::from_str::<Value>(text) else {
        return result;
    };
    if let Some(object) = body.as_object_mut() {
        object.insert("outcome".to_string(), outcome);
        if let Ok(serialized) = serde_json::to_string(&body) {
            *text = serialized;
        }
    }
    result
}

pub fn status(result: &CallToolResult) -> Option<OutcomeStatus> {
    let ToolContent::Text { text } = result.content.first()? else {
        return None;
    };
    let body: Value = serde_json::from_str(text).ok()?;
    match body.pointer("/outcome/status")?.as_str()? {
        "complete" => Some(OutcomeStatus::Complete),
        "partial" => Some(OutcomeStatus::Partial),
        "failed" => Some(OutcomeStatus::Failed),
        "uncertain" => Some(OutcomeStatus::Uncertain),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_preserves_legacy_fields_and_exposes_status() {
        let result = attach(
            CallToolResult::json(&json!({"placed_count": 2})),
            summary(
                OutcomeStatus::Partial,
                "board.kicad_sch",
                "saved_file_readback",
                3,
                2,
                1,
                Some(retry_failed_items(vec![1])),
            ),
        );
        let ToolContent::Text { text } = &result.content[0] else {
            panic!("expected text")
        };
        let body: Value = serde_json::from_str(text).unwrap();
        assert_eq!(body["placed_count"], 2);
        assert_eq!(body["outcome"]["status"], "partial");
        assert_eq!(status(&result), Some(OutcomeStatus::Partial));
    }
}
