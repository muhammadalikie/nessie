//! Optional, observational JSON Lines feedback for generated calls.
//!
//! This is **not** an oracle: ExecutionError is a generator-instrumentation
//! diagnostic, and a completed call need not have produced the intended result.

use crate::errors::{DFError, FunctionCallResult};
use crate::tests::{Test, TestDiagnostics};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct CallFeedback {
    pub call_id: String,
    pub function: String,
    pub access_path: Option<String>,
    pub is_constructor: bool,
    pub outcome: Option<FunctionCallResult>,
    pub callback_position: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestFeedback {
    pub schema_version: u32,
    pub test_id: usize,
    /// Number of execution attempts in this test-generation invocation.
    pub attempt: usize,
    /// `diagnosed` or `runner_error` (test subprocess output was not usable).
    pub execution: &'static str,
    /// Relevant only for runner_error; e.g. TestOutputParseError.
    pub runner_error: Option<String>,
    pub calls: Vec<CallFeedback>,
}

impl TestFeedback {
    pub fn diagnosed(test: &Test, attempt: usize, diagnostics: &TestDiagnostics) -> Self {
        Self::build(test, attempt, "diagnosed", None, Some(diagnostics))
    }

    pub fn runner_error(test: &Test, attempt: usize, error: &DFError) -> Self {
        Self::build(
            test,
            attempt,
            "runner_error",
            Some(format!("{:?}", error)),
            None,
        )
    }

    fn build(
        test: &Test,
        attempt: usize,
        execution: &'static str,
        runner_error: Option<String>,
        diagnostics: Option<&TestDiagnostics>,
    ) -> Self {
        let calls = test
            .get_fct_tree()
            .iter()
            .map(|node| {
                let fct = node.get();
                let node_id = test.get_fct_tree().get_node_id(node).unwrap();
                let (outcome, callback_position) = diagnostics
                    .and_then(|d| d.0.get(&node_id))
                    .map(|(result, position)| (Some(*result), position.clone()))
                    .unwrap_or((None, None));
                CallFeedback {
                    call_id: test.get_uniq_id_for_call(node),
                    function: fct.get_name().to_string(),
                    access_path: fct.get_acc_path().as_ref().map(ToString::to_string),
                    is_constructor: fct.is_constructor(),
                    outcome,
                    callback_position,
                }
            })
            .collect();
        Self {
            schema_version: 1,
            test_id: test.get_id(),
            attempt,
            execution,
            runner_error,
            calls,
        }
    }
}

/// Create a new trace; refuse to overwrite an existing trace accidentally.
pub struct FeedbackWriter {
    file: File,
}

impl FeedbackWriter {
    pub fn create(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        Ok(Self { file })
    }

    pub fn write(&mut self, record: &TestFeedback) -> io::Result<()> {
        serde_json::to_writer(&mut self.file, record)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        self.file.write_all(b"\n")?;
        self.file.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_is_valid_jsonl_with_a_versioned_schema() {
        let sample = TestFeedback {
            schema_version: 1,
            test_id: 9,
            attempt: 12,
            execution: "diagnosed",
            runner_error: None,
            calls: vec![CallFeedback {
                call_id: "2_pcid1_pos0".to_string(),
                function: "readJson".to_string(),
                access_path: Some("module.readJson".to_string()),
                is_constructor: false,
                outcome: Some(FunctionCallResult::ExecutionError),
                callback_position: None,
            }],
        };
        let val = serde_json::to_value(&sample).unwrap();
        assert_eq!(val["schema_version"], 1);
        assert_eq!(val["test_id"], 9);
        assert_eq!(val["calls"][0]["outcome"], "ExecutionError");
        assert_eq!(val["calls"][0]["call_id"], "2_pcid1_pos0");
    }
}
