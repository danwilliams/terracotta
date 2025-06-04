//! Terracotta
//! 
//! Boilerplate webserver application based on Axum.
//! 



//		Global configuration																							

//	Customisations of the standard linting configuration
#![allow(clippy::doc_markdown,            reason = "Too many false positives")]
#![allow(clippy::multiple_crate_versions, reason = "Cannot resolve all these")]

//	Lints specifically disabled for unit tests
#![cfg_attr(test, allow(
	non_snake_case,
	clippy::arithmetic_side_effects,
	clippy::cast_lossless,
	clippy::cast_precision_loss,
	clippy::cognitive_complexity,
	clippy::default_numeric_fallback,
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::expect_used,
	clippy::indexing_slicing,
	clippy::let_underscore_must_use,
	clippy::let_underscore_untyped,
	clippy::missing_assert_message,
	clippy::missing_panics_doc,
	clippy::must_use_candidate,
	clippy::panic,
	clippy::print_stdout,
	clippy::too_many_lines,
	clippy::unwrap_in_result,
	clippy::unwrap_used,
	reason = "Not useful in unit tests"
))]



//		Modules																											

pub mod app;
#[cfg(feature = "assets")]
pub mod assets;
#[cfg(feature = "auth")]
pub mod auth;
#[cfg(feature = "errors")]
pub mod errors;
#[cfg(feature = "health")]
pub mod health;
#[cfg(feature = "stats")]
pub mod stats;

/// List of crates used in the examples and not necessarily in the library.
#[cfg(test)]
mod examples {
	use parking_lot as _;
	use smart_default as _;
	#[cfg(not(windows))]
	use tikv_jemallocator as _;
}

/// List of crates used in feature-based tests and not necessarily in the
/// library.
#[cfg(test)]
mod feature_based_tests {
	use assert_json_diff as _;
	use rubedo as _;
	use serde_json as _;
}


