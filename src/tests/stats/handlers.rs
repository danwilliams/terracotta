#![allow(non_snake_case, reason = "To enable test function name organisation")]

//		Packages

use super::*;
use crate::{
	config::Config,
	stats::worker::{AppStateStats, AppStats, AppStatsTotals},
	utility::AppState,
};
use assert_json_diff::assert_json_eq;
use axum::{
	http::{Method, StatusCode},
	response::IntoResponse,
};
use chrono::Duration;
use core::sync::atomic::AtomicUsize;
use figment::{Figment, providers::Serialized};
use flume::{self};
use include_dir::include_dir;
use parking_lot::{Mutex, RwLock};
use rubedo::{
	http::{ResponseExt, UnpackedResponse, UnpackedResponseBody},
	sugar::s,
};
use tera::Tera;
use tokio::sync::broadcast;
use velcro::hash_map;



//		Functions

//		prepare_state															
fn prepare_state(start: NaiveDateTime) -> AppState {
	let (sender, _)     = flume::unbounded();
	let (tx, _)         = broadcast::channel(10);
	let mut state       = AppState {
		assets_dir:       Arc::new(include_dir!("static")),
		config:           Figment::from(Serialized::defaults(Config::default())).extract().unwrap(),
		content_dir:      Arc::new(include_dir!("content")),
		stats:            AppStateStats {
			data:                AppStats {
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
					times:       StatsForPeriod::default(),
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
					connections: StatsForPeriod::default(),
					memory:      StatsForPeriod::default(),
				}),
				..Default::default()
			},
			queue:               sender,
			broadcast:           tx,
		},
		template:         Tera::default(),
	};
	state.config.stats.periods = hash_map!{
		s!("second"):          1,
		s!("minute"):         60,
		s!("hour"):        3_600,
		s!("day"):        86_400,
	};
	state
}



//		Tests

//		stats																	
#[tokio::test]
async fn stats() {
	//	There is a very small possibility that this test will fail if the
	//	test is run at the exact moment that the date changes.
	let start           = Utc::now().naive_utc() - Duration::seconds(99);
	let state           = prepare_state(start);
	let unpacked        = get_stats(State(Arc::new(state))).await.into_response().unpack().unwrap();
	let crafted         = UnpackedResponse::new(
		StatusCode::OK,
		vec![
			//	Axum automatically adds a content-type header.
			(s!("content-type"), s!("application/json")),
		],
		UnpackedResponseBody::new(json!({
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
	);
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
		let mut buffers = state.stats.data.buffers.write();
		buffers.responses  .push_front(StatsForPeriod::default());
		buffers.connections.push_front(StatsForPeriod::default());
		buffers.memory     .push_front(StatsForPeriod::default());
	}
	let params       = GetStatsHistoryParams::default();
	let unpacked     = get_stats_history(State(Arc::new(state)), Query(params)).await.into_response().unpack().unwrap();
	let crafted      = UnpackedResponse::new(
		StatusCode::OK,
		vec![
			//	Axum automatically adds a content-type header.
			(s!("content-type"), s!("application/json")),
		],
		UnpackedResponseBody::new(json!({
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
	);
	assert_json_eq!(unpacked, crafted);
}


