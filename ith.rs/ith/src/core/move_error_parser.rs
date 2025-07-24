// Copyright 2020-2025 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Move error parsing utilities
//!
//! This module provides utilities to parse Move abort errors and extract
//! relevant information like abort codes and context.

use std::error::Error as StdError;

/// ITH Move contract error codes (from ith.move)
pub mod move_error_codes {
    /// Error when operation is performed with wrong federation
    pub const E_UNAUTHORIZED_WRONG_FEDERATION: u64 = 1;
    /// Error when entity lacks sufficient accreditation permissions
    pub const E_UNAUTHORIZED_INSUFFICIENT_ACCREDITATION_TO_ACCREDIT: u64 = 2;
    /// Error when entity lacks sufficient attestation permissions
    pub const E_UNAUTHORIZED_INSUFFICIENT_ACCREDITATION_TO_ATTEST: u64 = 3;
    /// Error when Value Condition for Statement is invalid
    pub const E_INVALID_STATEMENT_VALUE_CONDITION: u64 = 4;
    /// Error when trying to access non-existent accreditation
    pub const E_ACCREDITATION_NOT_FOUND: u64 = 5;
    /// Error when timestamp is in the past
    pub const E_TIMESTAMP_MUST_BE_IN_THE_FUTURE: u64 = 6;
}

/// Context information extracted from Move abort errors
#[derive(Debug, Clone)]
pub struct MoveErrorContext {
    pub module_name: Option<String>,
    pub function_name: Option<String>,
    pub function_index: Option<u64>,
    pub instruction_index: Option<u64>,
    pub full_error_string: String,
}

/// Trait for detecting and parsing Move contract errors
pub trait MoveErrorParser {
    /// Checks if the error is a Move abort error
    fn is_move_abort_error(&self) -> bool;

    /// Extracts the Move abort code if this is a Move abort error
    fn extract_move_abort_code(&self) -> Option<u64>;

    /// Extracts context information from the Move error (module, function, etc.)
    fn extract_move_error_context(&self) -> Option<MoveErrorContext>;
}

// ===== Default implementation for any std::error::Error =====

impl<T: StdError> MoveErrorParser for T {
    fn is_move_abort_error(&self) -> bool {
        let error_string = format!("{:?}", self);
        error_string.contains("MoveAbort(")
    }

    fn extract_move_abort_code(&self) -> Option<u64> {
        let error_string = format!("{:?}", self);
        extract_move_abort_code_from_string(&error_string)
    }

    fn extract_move_error_context(&self) -> Option<MoveErrorContext> {
        if !self.is_move_abort_error() {
            return None;
        }

        let error_string = format!("{:?}", self);
        Some(MoveErrorContext {
            module_name: extract_module_name(&error_string),
            function_name: extract_function_name(&error_string),
            function_index: extract_function_index(&error_string),
            instruction_index: extract_instruction_index(&error_string),
            full_error_string: error_string,
        })
    }
}

// ===== Helper functions for parsing error strings =====

/// Attempts to parse a Move abort error string and extract the abort code
///
/// Expected format: "MoveAbort(MoveLocation { ... }, abort_code)"
fn extract_move_abort_code_from_string(error_string: &str) -> Option<u64> {
    // Look for the pattern "}, <number>)" at the end of MoveAbort
    if let Some(move_abort_start) = error_string.find("MoveAbort(") {
        if let Some(abort_code_start) = error_string[move_abort_start..].rfind(", ") {
            let abort_section = &error_string[move_abort_start + abort_code_start + 2..];
            if let Some(abort_code_end) = abort_section.find(")") {
                let abort_code_str = &abort_section[..abort_code_end];
                return abort_code_str.trim().parse::<u64>().ok();
            }
        }
    }
    None
}

fn extract_module_name(error_string: &str) -> Option<String> {
    if let Some(name_start) = error_string.find("name: Identifier(\"") {
        let name_section = &error_string[name_start + 18..];
        if let Some(name_end) = name_section.find("\")") {
            return Some(name_section[..name_end].to_string());
        }
    }
    None
}

fn extract_function_name(error_string: &str) -> Option<String> {
    if let Some(func_start) = error_string.find("function_name: Some(\"") {
        let func_section = &error_string[func_start + 21..];
        if let Some(func_end) = func_section.find("\")") {
            return Some(func_section[..func_end].to_string());
        }
    }
    None
}

fn extract_function_index(error_string: &str) -> Option<u64> {
    if let Some(func_start) = error_string.find("function: ") {
        let func_section = &error_string[func_start + 10..];
        if let Some(func_end) = func_section.find(",") {
            return func_section[..func_end].trim().parse().ok();
        }
    }
    None
}

fn extract_instruction_index(error_string: &str) -> Option<u64> {
    if let Some(instr_start) = error_string.find("instruction: ") {
        let instr_section = &error_string[instr_start + 13..];
        if let Some(instr_end) = instr_section.find(",") {
            return instr_section[..instr_end].trim().parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use iota_sdk::types::execution_status::{ExecutionFailureStatus, ExecutionStatus};

    use super::*;

    #[test]
    fn test_extract_move_abort_code_from_string() {
        use iota_interaction::types::execution_status;
        let error_str = "MoveAbort(MoveLocation { module: ModuleId { address: 2319aae93cbe97ef111290154a571effb10989bca0ea8804e7804e8aebd9bb67, name: Identifier(\"main\") }, function: 13, instruction: 12, function_name: Some(\"add_statement\") }, 1)";

        let error_type = serde_json::from_str::<ExecutionFailureStatus>(error_str).unwrap();
        println!("{:?}", error_type);

        assert_eq!(extract_move_abort_code_from_string(error_str), Some(1));
    }

    #[test]
    fn test_extract_move_abort_code_different_numbers() {
        let error_str = "MoveAbort(MoveLocation { ... }, 5)";
        assert_eq!(extract_move_abort_code_from_string(error_str), Some(5));

        let error_str2 = "MoveAbort(MoveLocation { ... }, 999)";
        assert_eq!(extract_move_abort_code_from_string(error_str2), Some(999));
    }

    #[test]
    fn test_extract_move_abort_code_invalid() {
        assert_eq!(extract_move_abort_code_from_string("Not a move error"), None);
        assert_eq!(extract_move_abort_code_from_string("MoveAbort without numbers"), None);
    }

    #[test]
    fn test_extract_module_name() {
        let error_str = "MoveAbort(MoveLocation { module: ModuleId { address: 123, name: Identifier(\"main\") }, function: 13 }, 1)";
        assert_eq!(extract_module_name(error_str), Some("main".to_string()));
    }

    #[test]
    fn test_extract_function_name() {
        let error_str = "MoveAbort(MoveLocation { function_name: Some(\"add_statement\") }, 1)";
        assert_eq!(extract_function_name(error_str), Some("add_statement".to_string()));
    }

    #[test]
    fn test_extract_function_index() {
        let error_str = "MoveAbort(MoveLocation { function: 13, instruction: 12 }, 1)";
        assert_eq!(extract_function_index(error_str), Some(13));
    }

    #[test]
    fn test_extract_instruction_index() {
        let error_str = "MoveAbort(MoveLocation { function: 13, instruction: 12 }, 1)";
        assert_eq!(extract_instruction_index(error_str), Some(12));
    }

    // Test the trait implementation
    #[derive(Debug)]
    struct TestError {
        message: String,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for TestError {}

    #[test]
    fn test_move_error_parser_trait() {
        let move_error = TestError {
            message: "MoveAbort(MoveLocation { module: ModuleId { address: 123, name: Identifier(\"main\") }, function: 13, instruction: 12, function_name: Some(\"add_statement\") }, 5)".to_string(),
        };

        assert!(move_error.is_move_abort_error());
        assert_eq!(move_error.extract_move_abort_code(), Some(5));

        let context = move_error.extract_move_error_context().unwrap();
        assert_eq!(context.module_name, Some("main".to_string()));
        assert_eq!(context.function_name, Some("add_statement".to_string()));
        assert_eq!(context.function_index, Some(13));
        assert_eq!(context.instruction_index, Some(12));

        let non_move_error = TestError {
            message: "This is not a move error".to_string(),
        };

        assert!(!non_move_error.is_move_abort_error());
        assert_eq!(non_move_error.extract_move_abort_code(), None);
        assert!(non_move_error.extract_move_error_context().is_none());
    }
}
