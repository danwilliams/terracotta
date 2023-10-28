#![allow(non_snake_case)]

//		Tests

use super::super::*;
use axum::{
	http::StatusCode,
	response::IntoResponse,
};
use rubedo::{
	http::{ResponseExt, UnpackedResponse, UnpackedResponseBody, UnpackedResponseHeader},
	sugar::s,
};
use serde_json::json;

//		ping																	
#[tokio::test]
async fn ping() {
	let unpacked = get_ping().await.into_response().unpack().unwrap();
	let crafted  = UnpackedResponse {
		status:    StatusCode::OK,
		headers:   vec![],
		body:      UnpackedResponseBody::default(),
	};
	assert_eq!(unpacked, crafted);
}

//		version																	
#[tokio::test]
async fn version() {
	let unpacked = get_version().await.into_response().unpack().unwrap();
	let crafted  = UnpackedResponse {
		status:    StatusCode::OK,
		headers:       vec![
			//	Axum automatically adds a content-type header.
			UnpackedResponseHeader {
				name:  s!("content-type"),
				value: s!("application/json"),
			},
		],
		body:      UnpackedResponseBody::new(json!({
			"version": env!("CARGO_PKG_VERSION"),
		})),
	};
	assert_eq!(unpacked, crafted);
}


