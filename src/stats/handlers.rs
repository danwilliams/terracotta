#![allow(unused_qualifications, reason = "False positive")]
#![allow(clippy::unused_async,  reason = "Handler functions need to be async")]

//! Endpoint handlers for statistics functionality.



//		Modules

#[cfg(test)]
#[path = "../tests/stats/handlers.rs"]
mod tests;



//		Packages

use super::{
	requests::{GetStatsFeedParams, GetStatsHistoryParams, MeasurementType},
	responses::{StatsHistoryResponse, StatsResponse, StatsResponseForPeriod},
	state::StateProvider,
	worker::StatsForPeriod,
};
use axum::{
	Json,
	extract::{Query, State},
	extract::ws::{Message, WebSocketUpgrade, WebSocket},
	response::Response,
};
use chrono::{NaiveDateTime, SubsecRound, Utc};
use core::{
	sync::atomic::Ordering,
	time::Duration,
};
use indexmap::IndexMap;
use itertools::Itertools;
use rubedo::{
	std::IteratorExt,
	sugar::s,
};
use serde_json::json;
use std::{
	collections::{HashMap, VecDeque},
	sync::Arc,
	time::Instant,
};
use tokio::{
	select,
	time::interval,
};
use tracing::{info, warn};
use velcro::btree_map;



//		Functions

//		get_stats																
/// Application statistics overview.
/// 
/// This endpoint produces various statistics about the application. It returns
/// a JSON object containing the following information:
/// 
///   - `started_at`  - The date and time the application was started, in ISO
///                     8601 format.
///   - `last_second` - The latest second period that has been completed.
///   - `uptime`      - The amount of time the application has been running, in
///                     seconds.
///   - `requests`    - The number of requests that have been handled since the
///                     application last started.
///   - `active`      - The number of current open connections.
///   - `codes`       - The counts of responses that have been handled, broken
///                     down by status code, since the application last started.
///   - `times`       - The average, maximum, and minimum response times, plus
///                     sample count, for the [configured periods](super::config::Config#structfield.periods),
///                     and since the application last started.
///   - `endpoints`   - The counts of responses that have been handled, broken
///                     down by endpoint, since the application last started.
///   - `connections` - The average, maximum, and minimum open connections, plus
///                     sample count, for the [configured periods](super::config::Config#structfield.periods),
///                     and since the application last started.
///   - `memory`      - The average, maximum, and minimum memory usage, plus
///                     sample count, for the [configured periods](super::config::Config#structfield.periods),
///                     and since the application last started.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// 
#[utoipa::path(
	get,
	path = "/api/stats",
	tag  = "health",
	responses(
		(status = 200, description = "Application statistics overview", body = StatsResponse),
	)
)]
pub async fn get_stats<SP: StateProvider>(
	State(state): State<Arc<SP>>,
) -> Json<StatsResponse> {
	//		Helper functions													
	/// Initialises a map of stats for each period.
	fn initialize_map(
		periods: &HashMap<String, usize>,
		buffer:  &VecDeque<StatsForPeriod>,
	) -> IndexMap<String, StatsForPeriod> {
		let mut output: IndexMap<String, StatsForPeriod> = periods
			.iter()
			.sorted_by_key(|p| p.1)
			.map(|(name, _)| (name.clone(), StatsForPeriod::default()))
			.collect()
		;
		//	Loop through the circular buffer and calculate the stats
		for (i, stats) in buffer.iter().enumerate() {
			for (name, stats_for_period) in &mut output {
				if i < periods[name] {
					stats_for_period.update(stats);
				}
			}
		}
		output
	}
	
	/// Converts a map of stats for each period into response data.
	fn convert_map(
		input: IndexMap<String, StatsForPeriod>,
		all:   &StatsForPeriod
	) -> IndexMap<String, StatsResponseForPeriod> {
		let mut output: IndexMap<String, StatsResponseForPeriod> = input
			.into_iter()
			.map(|(key, value)| (key, StatsResponseForPeriod::from(&value)))
			.collect()
		;
		_ = output.insert(s!("all"), StatsResponseForPeriod::from(all));
		output
	}
	
	//		Preparation															
	//	Lock source data
	let stats_state  = state.state().read().await;
	let buffers      = stats_state.data.buffers.read();
	
	//	Create pots for each period and process stats buffers
	let timing_input = initialize_map(&state.config().periods, &buffers.responses);
	let conn_input   = initialize_map(&state.config().periods, &buffers.connections);
	let memory_input = initialize_map(&state.config().periods, &buffers.memory);
	
	//	Unlock source data
	drop(buffers);
	
	//		Process stats														
	//	Lock source data
	let totals        = stats_state.data.totals.lock();
	
	//	Convert the input stats data into the output stats data
	let timing_output = convert_map(timing_input, &totals.times);
	let conn_output   = convert_map(conn_input,   &totals.connections);
	let memory_output = convert_map(memory_input, &totals.memory);
	
	//		Build response data													
	let now        = Utc::now().naive_utc();
	#[expect(clippy::arithmetic_side_effects, reason = "Nothing interesting can happen here")]
	#[expect(clippy::cast_sign_loss,          reason = "We don't ever want a negative for uptime")]
	let response   = Json(StatsResponse {
		started_at:  stats_state.data.started_at.trunc_subsecs(0),
		last_second: *stats_state.data.last_second.read(),
		uptime:      (now - stats_state.data.started_at).num_seconds() as u64,
		active:      stats_state.data.connections.load(Ordering::Relaxed) as u64,
		requests:    stats_state.data.requests.load(Ordering::Relaxed) as u64,
		codes:       totals.codes.clone(),
		times:       timing_output,
		endpoints:   totals.endpoints.iter()
			.map(|(key, value)| (key.clone(), StatsResponseForPeriod::from(value)))
			.collect()
		,
		connections: conn_output,
		memory:      memory_output,
	});
	//	Unlock source data
	drop(totals);
	drop(stats_state);
	
	//		Response															
	response
}

//		get_stats_history														
/// Historical application statistics interval data.
/// 
/// This endpoint provides access to historical application statistics interval
/// data available from the statistics buffers. It returns a JSON object
/// containing the following information:
/// 
///   - `last_second` - The latest second period that has been completed.
///   - `times`       - The average, maximum, and minimum response times, plus
///                     sample count, per second for every second since the
///                     application last started, or up until the end of the
///                     [configured buffer](super::config::Config#structfield.timing_buffer_size).
///   - `connections` - The average, maximum, and minimum open connections, plus
///                     sample count, per second for every second since the
///                     application last started, or up until the end of the
///                     [configured buffer](super::config::Config#structfield.connection_buffer_size).
///   - `memory`      - The average, maximum, and minimum memory usage, plus
///                     sample count, per second for every second since the
///                     application last started, or up until the end of the
///                     [configured buffer](super::config::Config#structfield.memory_buffer_size).
/// 
/// # Parameters
/// 
/// * `state`  - The application state.
/// * `params` - The parameters for the request.
/// 
#[utoipa::path(
	get,
	path = "/api/stats/history",
	tag  = "health",
	params(
		GetStatsHistoryParams,
	),
	responses(
		(status = 200, description = "Historical application statistics interval data", body = StatsHistoryResponse),
	)
)]
pub async fn get_stats_history<SP: StateProvider>(
	State(state):  State<Arc<SP>>,
	Query(params): Query<GetStatsHistoryParams>,
) -> Json<StatsHistoryResponse> {
	//		Helper function														
	/// Processes a buffer of statistics data.
	fn process_buffer(
		buffer: &VecDeque<StatsForPeriod>,
		from:   Option<NaiveDateTime>,
		limit:  Option<usize>,
	) -> Vec<StatsResponseForPeriod> {
		buffer.iter()
			.take_while(|entry| from.map_or(true, |time| entry.started_at >= time))
			.limit(limit)
			.map(StatsResponseForPeriod::from)
			.collect()
	}
	
	//		Prepare response data												
	//	Lock source data
	let stats_state  = state.state().read().await;
	let buffers      = stats_state.data.buffers.read();
	let mut response = StatsHistoryResponse {
		last_second:   *stats_state.data.last_second.read(),
		..Default::default()
	};
	//	Convert the statistics buffers
	match params.buffer {
		Some(MeasurementType::Times)       => {
			response.times       = process_buffer(&buffers.responses,   params.from, params.limit);
		},
		Some(MeasurementType::Connections) => {
			response.connections = process_buffer(&buffers.connections, params.from, params.limit);
		},
		Some(MeasurementType::Memory)      => {
			response.memory      = process_buffer(&buffers.memory,      params.from, params.limit);
		},
		None                               => {
			response.times       = process_buffer(&buffers.responses,   params.from, params.limit);
			response.connections = process_buffer(&buffers.connections, params.from, params.limit);
			response.memory      = process_buffer(&buffers.memory,      params.from, params.limit);
		},
	}
	//	Unlock source data
	drop(buffers);
	drop(stats_state);
	Json(response)
}

//		get_stats_feed															
/// Application statistics event feed.
/// 
/// This endpoint returns an open WebSocket connection for a feed of statistics
/// events. It will establish a handshake with the [`WebSocket`] and then pass
/// over to [`ws_stats_feed()`] to handle the connection. This function will
/// then return a [`Response`] with a status code of `101 Switching Protocols`
/// and the `Connection` header set to `Upgrade`.
/// 
/// # Parameters
/// 
/// * `state`  - The application state.
/// * `params` - The parameters for the request.
/// * `ws_req` - The websocket request.
/// 
#[utoipa::path(
	get,
	path = "/api/stats/feed",
	tag  = "health",
	params(
		GetStatsFeedParams,
	),
	responses(
		(status = 200, description = "Application statistics event feed"),
	),
)]
pub async fn get_stats_feed<SP: StateProvider>(
	State(state):  State<Arc<SP>>,
	Query(params): Query<GetStatsFeedParams>,
	ws_req:        WebSocketUpgrade,
) -> Response {
	//	Establish a handshake with the WebSocket
	ws_req.on_upgrade(move |socket| ws_stats_feed(Arc::clone(&state), socket, params.r#type))
}

//		ws_stats_feed															
/// WebSocket feed of application statistics events.
/// 
/// This endpoint returns a feed of application statistics over a WebSocket
/// connection established by [`get_stats_feed()`]. Statistics events are sent
/// as they are received from the broadcast channel. The events are
/// [`StatsForPeriod`] instances, sent as JSON objects.
/// 
/// Notably, if not filtered by measurement type, all measurement types will
/// have their statistics returned in a JSON object, with the type names as keys
/// and the statistics data in sub-objects. However, when filtered by type, only
/// the statistics object for that one type will be returned. This is in order
/// to keep the transmitted data as efficient as possible.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// * `ws`    - The websocket stream.
/// * `scope` - The type of measurement statistics to send.
/// 
#[expect(clippy::similar_names, reason = "Clearly different")]
pub async fn ws_stats_feed<SP: StateProvider>(
	state:  Arc<SP>,
	mut ws: WebSocket,
	scope:  Option<MeasurementType>,
) {
	//		Preparation															
	info!("WebSocket connection established");
	//	Subscribe to the broadcast channel
	#[expect(clippy::significant_drop_in_scrutinee, reason = "Short-lived")]
	let mut rx        = if let Some(ref broadcaster) = state.state().read().await.broadcaster {
		broadcaster.subscribe()
	} else {
		warn!("Broadcast channel not available");
		return;
	};
	//	Set up a timer to send pings at regular intervals
	let mut timer     = interval(Duration::from_secs(state.config().ws_ping_interval as u64));
	let mut timeout   = interval(Duration::from_secs(state.config().ws_ping_timeout  as u64));
	let mut last_ping = None;
	let mut last_pong = Instant::now();
	
	//	Message processing loop
	#[expect(clippy::pattern_type_mismatch, reason = "Tokio code")]
	loop { select! {
		//		Ping															
		//	Send a ping at regular intervals
		_ = timer.tick() => {
			if let Err(err) = ws.send(Message::Ping(Vec::new())).await {
				warn!("Failed to send ping over WebSocket: {err}");
				break;
			}
			last_ping = Some(Instant::now());
		},
		//		Ping/pong timeout												
		//	Check for ping timeout (X seconds since the last ping without a pong)
		_ = timeout.tick() => {
			if let Some(ping_time) = last_ping {
				let limit = Duration::from_secs(state.config().ws_ping_timeout as u64);
				if last_pong < ping_time && ping_time.elapsed() > limit {
					warn!("WebSocket ping timed out");
					break;
				}
			}
		},
		//		Incoming message												
		//	Handle incoming messages from the WebSocket
		Some(msg) = ws.recv() => {
			match msg {
				Ok(Message::Ping(ping)) => {
					if let Err(err) = ws.send(Message::Pong(ping)).await {
						warn!("Failed to send pong over WebSocket: {err}");
						break;
					}
				}
				Ok(Message::Pong(_))    => {
					last_pong = Instant::now();
				}
				Ok(Message::Close(_))   => {
					info!("WebSocket connection closed");
					break;
				}
				Ok(Message::Text(_))    => {
					warn!("Unexpected WebSocket text message");
				}
				Ok(Message::Binary(_))  => {
					warn!("Unexpected WebSocket binary message");
				}
				Err(err)                => {
					warn!("WebSocket error: {err}");
					break;
				}
				#[expect(unreachable_patterns, reason = "Future-proofing")]
				_                       => {
					//	At present there are no other message types, but this is here to catch
					//	any future additions.
					warn!("Unknown WebSocket message type");
				}
			}	
		}
		//		Send stats data													
		//	Handle new data from the broadcast channel
		Ok(data) = rx.recv() => {
			let response = match scope {
				Some(MeasurementType::Times)       => {
					json!{StatsResponseForPeriod::from(&data.times)}
				},
				Some(MeasurementType::Connections) => {
					json!{StatsResponseForPeriod::from(&data.connections)}
				},
				Some(MeasurementType::Memory)      => {
					json!{StatsResponseForPeriod::from(&data.memory)}
				},
				None                               => {
					json!{btree_map!{
						"times":       StatsResponseForPeriod::from(&data.times),
						"connections": StatsResponseForPeriod::from(&data.connections),
						"memory":      StatsResponseForPeriod::from(&data.memory),
					}}
				},
			};
			if let Err(err) = ws.send(Message::Text(response.to_string())).await {
				warn!("Failed to send data over WebSocket: {err}");
				break;
			}
		}
	}}
}


