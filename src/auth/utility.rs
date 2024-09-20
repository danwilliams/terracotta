//! Utility functions for authentication.



//		Packages

use axum::http::{
	Uri,
	uri::InvalidUriParts as RealInvalidUriParts,
};
use core::{
	error::Error,
	fmt::{Display, Formatter, self},
	hash::BuildHasher,
};
use std::collections::HashMap;
use tracing::warn;
use url::form_urlencoded;



//		Structs

//		InvalidUriParts															
/// Represents an error when constructing a URI from parts.
/// 
/// This type exists because the [`Builder::build()`](axum::http::uri::Builder::build())
/// method returns an [`HttpError`](axum::http::Error) but this covers more
/// possibilities than just an invalid URI — and the inner error type is not
/// cloneable or reconstructable. Therefore this type is used to represent just
/// the invalid URI case.
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidUriParts(String);

//󰭅		Display																	
impl Display for InvalidUriParts {
	//		fmt																	
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "Invalid URI parts: {}", self.0)
	}
}

//󰭅		Error																	
impl Error for InvalidUriParts {}



//		Functions

//		extract_uri_query_parts													
/// Extracts the query parts from a URI.
/// 
/// Extracts the query parts of a [`Uri`] and returns them as a [`HashMap`].
/// 
/// # Parameters
/// 
/// * `uri` - The URI to extract the query parts from.
/// 
pub fn extract_uri_query_parts(uri: &Uri) -> HashMap<String, String> {
	uri
		.query()
		.map(|v| form_urlencoded::parse(v.as_bytes()).into_owned().collect())
		.unwrap_or_default()
}

//		build_uri																
/// Builds a URI from a path and a set of query parameters.
/// 
/// # Parameters
/// 
/// * `path`   - The path to build the URI from.
/// * `params` - The query parameters to add to the URI.
/// 
/// # Errors
/// 
/// If the URI cannot be built, an error will be returned.
/// 
pub fn build_uri<P, K, V, H>(path: P, params: &HashMap<K, V, H>) -> Result<Uri, InvalidUriParts>
where
	P: AsRef<str>,
	K: AsRef<str> + Display,
	V: AsRef<str> + Display,
	H: BuildHasher,
{
	Uri::builder()
		.path_and_query(format!(
			"{}?{}",
			path.as_ref(),
			params
				.iter()
				.map(|(k, v)| format!("{k}={v}"))
				.collect::<Vec<String>>()
				.join("&")
			,
		))
		.build()
		.map_err(|err| {
			if !err.is::<RealInvalidUriParts>() {
				//	This is not expected, and if it does occur, we are still capturing the
				//	error message, so the behaviour is fine - but it's worth logging.
				warn!("Expected InvalidUriParts, but got a different error type: {err}");
			}
			InvalidUriParts(err.to_string())
		})
}


