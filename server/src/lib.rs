// Pre-existing clippy lints from generated DSL code and legacy patterns.
// Fixing them individually is churn because the root cause lives in
// proc-macro output or very repetitive seed data. The allow-list keeps
// the build warning-free so new warnings are immediately visible.
#![allow(
    clippy::redundant_field_names,
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::useless_conversion,
    clippy::decimal_literal_representation,
    clippy::filter_next,
    clippy::match_single_binding,
    clippy::redundant_pattern_matching,
    clippy::ok_expect,
    clippy::needless_borrow,
    clippy::assign_op_pattern,
    clippy::manual_clamp,
    clippy::unnecessary_cast,
    clippy::to_string_in_format_args,
    clippy::type_complexity,
    clippy::format_in_format_args,
    clippy::legacy_numeric_constants,
    clippy::unwrap_used,
)]

use spacetimedsl::*;
use tables::*;

pub mod admin;
pub mod definitions;
pub mod logic;
pub mod tables;
pub mod utility;

pub mod lifecycle;
