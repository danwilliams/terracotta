#![allow(non_snake_case)]

//		Tests

use super::*;
use crate::utility::Config;
use assert_json_diff::assert_json_eq;
use axum::{
	http::{Method, StatusCode},
	response::IntoResponse,
};
use chrono::Duration;
use figment::{Figment, providers::Serialized};
use flume::{self};
use parking_lot::Mutex;
use rand::Rng;
use ring::hmac::{HMAC_SHA512, self};
use rubedo::{
	http::{ResponseExt, UnpackedResponse, UnpackedResponseBody, UnpackedResponseHeader},
	sugar::s,
};
use std::sync::atomic::AtomicUsize;
use tera::Tera;
use tokio::sync::broadcast;
use velcro::hash_map;

//		prepare_state															
fn prepare_state(start: NaiveDateTime) -> AppState {
	let (sender, _)     = flume::unbounded();
	let (tx, _)         = broadcast::channel(10);
	let secret          = rand::thread_rng().gen::<[u8; 64]>();
	let mut state       = AppState {
		Config:           Figment::from(Serialized::defaults(Config::default())).extract().unwrap(),
		Stats:            AppStateStats {
			Data:                AppStats {
				started_at:      start,
				last_second:     RwLock::new((start + Duration::seconds(95)).with_nanosecond(0).unwrap()),
				connections:     AtomicUsize::new(5),
				requests:        AtomicUsize::new(10),
				totals:          Mutex::new(AppStatsTotals {
					codes:       hash_map!{
						StatusCode::OK:                    5,
						StatusCode::UNAUTHORIZED:          4,
						StatusCode::NOT_FOUND:             3,
						StatusCode::INTERNAL_SERVER_ERROR: 2,
					},
					times:       Default::default(),
					endpoints:   hash_map!{
						Endpoint {
							method:     Method::GET,
							path:       s!("/api/stats"),
						}:              StatsForPeriod {
							started_at: start,
							average:    500.0,
							maximum:    1000,
							minimum:    100,
							count:      10,
						},
					},
					connections: Default::default(),
					memory:      Default::default(),
				}),
				..Default::default()
			},
			Queue:               sender,
			Broadcast:           tx,
		},
		Secret:           secret,
		Key:              hmac::Key::new(HMAC_SHA512, &secret),
		Template:         Tera::default(),
	};
	state.Config.stats_periods = hash_map!{
		s!("second"):          1,
		s!("minute"):         60,
		s!("hour"):        3_600,
		s!("day"):        86_400,
	};
	state
}

//		stats																	
#[tokio::test]
async fn stats() {
	//	There is a very small possibility that this test will fail if the
	//	test is run at the exact moment that the date changes.
	let start           = Utc::now().naive_utc() - Duration::seconds(99);
	let state           = prepare_state(start);
	let unpacked        = get_stats(State(Arc::new(state))).await.into_response().unpack().unwrap();
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
			"started_at":  start.with_nanosecond(0).unwrap(),
			"last_second": (start + Duration::seconds(95)).with_nanosecond(0).unwrap(),
			"uptime":      99,
			"active":      5,
			"requests":    10,
			"codes":                         {
				"200 OK":                    5,
				"401 Unauthorized":          4,
				"404 Not Found":             3,
				"500 Internal Server Error": 2,
			},
			"times":  {
				"second":      {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"minute":      {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"hour":        {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"day":         {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"all":         {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			},
			"endpoints": {
				"GET /api/stats": {
					"average":    500.0,
					"maximum":    1000,
					"minimum":    100,
					"count":      10,
				},
			},
			"connections": {
				"second":      {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"minute":      {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"hour":        {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"day":         {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"all":         {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			},
			"memory": {
				"second":      {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"minute":      {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"hour":        {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"day":         {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"all":         {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			},
		})),
	};
	assert_json_eq!(unpacked, crafted);
}

//		stats_history															
#[tokio::test]
async fn stats_history() {
	//	There is a very small possibility that this test will fail if the
	//	test is run at the exact moment that the date changes.
	let start        = Utc::now().naive_utc() - Duration::seconds(99);
	let state        = prepare_state(start);
	{
		let mut buffers = state.Stats.Data.buffers.write();
		buffers.responses  .push_front(StatsForPeriod::default());
		buffers.connections.push_front(StatsForPeriod::default());
		buffers.memory     .push_front(StatsForPeriod::default());
	}
	let params       = GetStatsHistoryParams::default();
	let unpacked     = get_stats_history(State(Arc::new(state)), Query(params)).await.into_response().unpack().unwrap();
	let crafted      = UnpackedResponse {
		status:        StatusCode::OK,
		headers:       vec![
			//	Axum automatically adds a content-type header.
			UnpackedResponseHeader {
				name:  s!("content-type"),
				value: s!("application/json"),
			},
		],
		body:          UnpackedResponseBody::new(json!({
			"last_second":     (start + Duration::seconds(95)).with_nanosecond(0).unwrap(),
			"times": [
				{
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			],
			"connections": [
				{
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			],
			"memory": [
				{
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			],
		})),
	};
	assert_json_eq!(unpacked, crafted);
}


