//! Utility functions for statistics processing.



//		Packages

use axum::http::StatusCode;
use core::hash::BuildHasher;
use serde::{Serialize as _, Serializer};
use std::collections::{BTreeMap, HashMap};



//		Functions

//		serialize_status_codes													
/// Returns a list of serialised status code entries and their values.
/// 
/// This function is used by [`serde`] to serialise a list of status codes and
/// their associated values. It returns the list sorted by status code.
/// 
/// # Parameters
/// 
/// * `status_codes` - The status codes to serialise, as keys, against values.
/// * `serializer`   - The serialiser to use.
/// 
/// # Errors
/// 
/// If there is an error serialising the status codes, an error will be
/// returned.
/// 
pub fn serialize_status_codes<S, H>(
	status_codes: &HashMap<StatusCode, u64, H>,
	serializer:   S,
) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	H: BuildHasher,
{
	let codes: BTreeMap<String, u64> = status_codes
		.iter()
		.map(|(key, value)| (key.to_string(), *value))
		.collect()
	;
	codes.serialize(serializer)
}


