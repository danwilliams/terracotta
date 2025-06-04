#![allow(non_snake_case, reason = "To enable test function name organisation")]

//		Packages

use super::*;
use axum::{
	http::StatusCode,
	response::IntoResponse as _,
};
use rubedo::{
	http::{ResponseExt as _, UnpackedResponse, UnpackedResponseBody},
	sugar::s,
};
use serde_json::json;



//		Tests

//		ping																	
#[expect(clippy::unit_arg, reason = "Needed for the test")]
#[tokio::test]
async fn ping() {
	let unpacked = get_ping().await.into_response().unpack().unwrap();
	let crafted  = UnpackedResponse::new(
		StatusCode::OK,
		vec![],
		UnpackedResponseBody::default(),
	);
	assert_eq!(unpacked, crafted);
}

//		version																	
#[tokio::test]
async fn version() {
	let unpacked = get_version().await.into_response().unpack().unwrap();
	let crafted  = UnpackedResponse::new(
		StatusCode::OK,
		vec![
			//	Axum automatically adds a content-type header.
			(s!("content-type"), s!("application/json")),
		],
		UnpackedResponseBody::new(json!({
			"version": env!("CARGO_PKG_VERSION"),
		})),
	);
	assert_eq!(unpacked, crafted);
}


