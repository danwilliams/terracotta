#![allow(non_snake_case)]

//		Tests

//		healthcheck																
#[cfg(test)]
mod healthcheck {
	use super::super::*;
	use crate::utility::{AppStats, Config};
	use assert_json_diff::assert_json_eq;
	use axum::http::StatusCode;
	use chrono::Duration;
	use figment::{Figment, providers::Serialized};
	use rand::Rng;
	use ring::hmac::{HMAC_SHA512, self};
	use rubedo::{
		http::{ResponseExt, UnpackedResponse, UnpackedResponseBody, UnpackedResponseHeader},
		sugar::s,
	};
	use serde_json::json;
	use tera::Tera;
	
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
	
	//		stats																
	#[tokio::test]
	async fn stats() {
		//	There is a very small possibility that this test will fail if the
		//	test is run at the exact moment that the date changes.
		let start           = Utc::now().naive_utc() - Duration::seconds(99);
		let secret          = rand::thread_rng().gen::<[u8; 64]>();
		let state           = Arc::new(AppState {
			Config:           Figment::from(Serialized::defaults(Config::default())).extract().unwrap(),
			Stats:            AppStats {
				started_at:   start,
				..Default::default()
			},
			Secret:           secret,
			Key:              hmac::Key::new(HMAC_SHA512, &secret),
			Template:         Tera::default(),
		});
		let unpacked        = get_stats(State(state)).await.into_response().unpack().unwrap();
		let crafted         = UnpackedResponse {
			status:           StatusCode::OK,
			headers:          vec![
				//	Axum automatically adds a content-type header.
				UnpackedResponseHeader {
					name:     s!("content-type"),
					value:    s!("application/json"),
				},
			],
			body:             UnpackedResponseBody::new(json!({
				"started_at": start,
				"uptime":     99,
			})),
		};
		assert_json_eq!(unpacked, crafted);
	}
}


