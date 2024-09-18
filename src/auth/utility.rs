//! Utility functions for authentication.



//		Packages

use axum::http::Uri;
use core::fmt::Display;
use std::collections::HashMap;
use url::form_urlencoded;



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
		.map(|v| {
			form_urlencoded::parse(v.as_bytes())
				.into_owned()
				.collect()
		})
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
pub fn build_uri<P, K, V>(path: P, params: &HashMap<K, V>) -> Uri
where
	P: AsRef<str>,
	K: AsRef<str> + Display,
	V: AsRef<str> + Display,
{
	Uri::builder()
		.path_and_query(format!("{}?{}",
			path.as_ref(),
			params
				.iter()
				.map(|(k, v)| format!("{k}={v}"))
				.collect::<Vec<String>>()
				.join("&")
			,
		))
		.build()
		.unwrap()
}


