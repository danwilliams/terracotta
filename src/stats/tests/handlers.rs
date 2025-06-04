#![allow(non_snake_case, reason = "To enable test function name organisation")]

//		Packages

use super::*;
use super::super::{
	config::Config as StatsConfig,
	state::{State, StateProvider, Stats, StatsTotals},
	worker::Endpoint,
};
use assert_json_diff::assert_json_eq;
use axum::{
	http::{Method, StatusCode},
	response::IntoResponse as _,
};
use chrono::{TimeDelta, SubsecRound as _};
use core::sync::atomic::AtomicUsize;
use figment::{Figment, providers::Serialized};
use parking_lot::{Mutex, RwLock};
use rubedo::{
	http::{ResponseExt as _, UnpackedResponse, UnpackedResponseBody},
	sugar::s,
};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use tokio::sync::RwLock as AsyncRwLock;
use velcro::hash_map;



//		Structs

//		Config																	
/// The main configuration options for the application.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
struct Config {
	//		Public properties													
	/// The configuration options for gathering and processing statistics.
	pub stats: StatsConfig,
}

//		AppState																
/// The application state.
/// 
/// This is used to store global state information that is shared between
/// requests.
/// 
#[derive(Debug)]
struct AppState {
	//		Public properties													
	/// The application configuration.
	pub config: Config,
	
	/// The application statistics.
	pub stats:  AsyncRwLock<State>,
}

//󰭅		StateProvider															
impl StateProvider for AppState {
	//		config																
	fn config(&self) -> &StatsConfig {
		&self.config.stats
	}
	
	//		state																
	fn state(&self) -> &AsyncRwLock<State> {
		&self.stats
	}
}



//		Functions

//		prepare_state															
fn prepare_state(start: NaiveDateTime) -> AppState {
	let mut state = AppState {
		config: Figment::from(Serialized::defaults(Config::default())).extract().unwrap(),
		stats:  AsyncRwLock::new(State {
			data:        Stats {
				started_at:  start,
				last_second: RwLock::new((start + TimeDelta::seconds(95)).trunc_subsecs(0)),
				connections: AtomicUsize::new(5),
				requests:    AtomicUsize::new(10),
				totals:      Mutex::new(StatsTotals {
					codes:       hash_map!{
						StatusCode::OK:                    5,
						StatusCode::UNAUTHORIZED:          4,
						StatusCode::NOT_FOUND:             3,
						StatusCode::INTERNAL_SERVER_ERROR: 2,
					},
					times:       StatsForPeriod::default(),
					endpoints:   hash_map!{
						Endpoint {
							method: Method::GET,
							path:   s!("/api/stats"),
						}: StatsForPeriod {
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
			queue:       None,
			broadcaster: None,
			listener:    None,
		}),
	};
	state.config.stats.periods = hash_map!{
		s!("second"):      1,
		s!("minute"):     60,
		s!("hour"):    3_600,
		s!("day"):    86_400,
	};
	state
}



//		Tests

//		stats																	
#[tokio::test]
async fn stats() {
	//	There is a very small possibility that this test will fail if the
	//	test is run at the exact moment that the date changes.
	let start    = Utc::now().naive_utc() - TimeDelta::seconds(99);
	let state    = prepare_state(start);
	let unpacked = get_stats(State(Arc::new(state))).await.into_response().unpack().unwrap();
	let crafted  = UnpackedResponse::new(
		StatusCode::OK,
		vec![
			//	Axum automatically adds a content-type header.
			(s!("content-type"), s!("application/json")),
		],
		UnpackedResponseBody::new(json!({
			"started_at":  start.trunc_subsecs(0),
			"last_second": (start + TimeDelta::seconds(95)).trunc_subsecs(0),
			"uptime":      99,
			"active":      5,
			"requests":    10,
			"codes": {
				"200 OK":                    5,
				"401 Unauthorized":          4,
				"404 Not Found":             3,
				"500 Internal Server Error": 2,
			},
			"times": {
				"second": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"minute": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"hour": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"day": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"all": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			},
			"endpoints": {
				"GET /api/stats": {
					"average": 500.0,
					"maximum": 1000,
					"minimum": 100,
					"count":   10,
				},
			},
			"connections": {
				"second": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"minute": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"hour": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"day": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"all": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
			},
			"memory": {
				"second": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"minute": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"hour": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"day": {
					"average": 0.0,
					"maximum": 0,
					"minimum": 0,
					"count":   0,
				},
				"all": {
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
	let start = Utc::now().naive_utc() - TimeDelta::seconds(99);
	let state = prepare_state(start);
	{
		let stats_state = state.stats.read().await;
		let mut buffers = stats_state.data.buffers.write();
		buffers.responses  .push_front(StatsForPeriod::default());
		buffers.connections.push_front(StatsForPeriod::default());
		buffers.memory     .push_front(StatsForPeriod::default());
		drop(buffers);
		drop(stats_state);
	}
	let params   = GetStatsHistoryParams::default();
	let unpacked = get_stats_history(State(Arc::new(state)), Query(params)).await.into_response().unpack().unwrap();
	let crafted  = UnpackedResponse::new(
		StatusCode::OK,
		vec![
			//	Axum automatically adds a content-type header.
			(s!("content-type"), s!("application/json")),
		],
		UnpackedResponseBody::new(json!({
			"last_second": (start + TimeDelta::seconds(95)).trunc_subsecs(0),
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


