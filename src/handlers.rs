//		Modules

#[cfg(test)]
#[path = "tests/handlers.rs"]
mod tests;



//		Packages

use crate::{
	ASSETS_DIR,
	CONTENT_DIR,
	utility::*,
};
use axum::{
	Json,
	body::Body,
	extract::State,
	http::{HeaderValue, StatusCode, Uri, header},
	response::{Html, IntoResponse, Response},
};
use chrono::{NaiveDateTime, Utc};
use indexmap::IndexMap;
use itertools::Itertools;
use mime_guess::{self};
use rubedo::sugar::s;
use serde::Serialize;
use std::{
	collections::{HashMap, VecDeque},
	sync::{Arc, atomic::Ordering},
};
use tera::Context;
use tokio::{
	fs::File,
	io::{AsyncReadExt, BufReader},
};
use tokio_util::io::ReaderStream;
use utoipa::ToSchema;



//		Enums

//		AssetContext															
/// The protection contexts for static assets.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AssetContext {
	/// Public files.
	Public,
	
	/// Protected files.
	Protected,
}



//		Structs

//		StatsResponse															
/// The application statistics returned by the `/api/stats` endpoint.
#[derive(Serialize, ToSchema)]
pub struct StatsResponse {
	//		Public properties													
	/// The date and time the application was started.
	pub started_at:  NaiveDateTime,
	
	/// The amount of time the application has been running, in seconds.
	pub uptime:      u64,
	
	/// The current number of open connections, i.e. requests that have not yet
	/// been responded to.
	pub active:      u64,
	
	/// The number of requests that have been made. The number of responses will
	/// be incremented only when the request has been fully handled and a
	/// response generated.
	pub requests:    u64,
	
	/// The number of responses that have been handled, by status code.
	#[serde(serialize_with = "serialize_status_codes")]
	pub codes:       HashMap<StatusCode, u64>,
	
	/// The average, maximum, and minimum response times in microseconds, plus
	/// sample count, grouped by time period.
	pub times:       IndexMap<String, StatsResponseForPeriod>,
	
	/// The average, maximum, and minimum response times in microseconds, plus
	/// sample count, grouped by endpoint, since the application last started.
	pub endpoints:   HashMap<Endpoint, StatsResponseForPeriod>,
	
	/// The average, maximum, and minimum open connections, plus sample count,
	/// grouped by time period.
	pub connections: IndexMap<String, StatsResponseForPeriod>,
	
	/// The average, maximum, and minimum memory usage in bytes, plus sample
	/// count, grouped by time period.
	pub memory:      IndexMap<String, StatsResponseForPeriod>,
}

//		StatsResponseForPeriod													
/// Average, maximum, minimum, and count of values for a period of time.
#[derive(Serialize, ToSchema)]
pub struct StatsResponseForPeriod {
	//		Public properties													
	/// Average value.
	pub average: f64,
	
	/// Maximum value.
	pub maximum: u64,
	
	/// Minimum value.
	pub minimum: u64,
	
	/// The total number of values.
	pub count:   u64,
}

impl From<&StatsForPeriod> for StatsResponseForPeriod {
	//		from																
	fn from(stats: &StatsForPeriod) -> Self {
		Self {
			average: stats.average,
			maximum: stats.maximum,
			minimum: stats.minimum,
			count:   stats.count,
		}
	}
}



//		Functions

//		get_index																
/// Shows the index page.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// 
pub async fn get_index(State(state): State<Arc<AppState>>) -> Html<String> {
	let mut context = Context::new();
	context.insert("Title",   &state.Config.title);
	context.insert("Content", "Index");
	Html(state.Template.render("index", &context).unwrap())
}

//		get_protected_static_asset												
/// Serves protected static assets.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// * `uri`   - The URI of the asset.
/// 
pub async fn get_protected_static_asset(
	State(state): State<Arc<AppState>>,
	uri:          Uri,
) -> impl IntoResponse {
	get_static_asset(state, uri, AssetContext::Protected).await
}

//		get_public_static_asset													
/// Serves public static assets.
/// 
/// # Parameters
/// 
/// * `state` - The application state.
/// * `uri`   - The URI of the asset.
/// 
pub async fn get_public_static_asset(
	State(state): State<Arc<AppState>>,
	uri:          Uri,
) -> impl IntoResponse {
	get_static_asset(state, uri, AssetContext::Public).await
}

//		get_static_asset														
/// Serves static assets.
/// 
/// # Parameters
/// 
/// * `state`   - The application state.
/// * `uri`     - The URI of the asset.
/// * `context` - The protection context of the asset to serve.
/// 
async fn get_static_asset(
	state:   Arc<AppState>,
	uri:     Uri,
	context: AssetContext
) -> impl IntoResponse {
	let path       =  uri.path().trim_start_matches('/');
	let mime_type  =  mime_guess::from_path(path).first_or_text_plain();
	let (basedir, local_path, behavior) = match context {
		AssetContext::Public    => (
			&ASSETS_DIR,
			state.Config.local_paths.public_assets.join(path),
			&state.Config.local_loading.public_assets
		),
		AssetContext::Protected => (
			&CONTENT_DIR,
			state.Config.local_paths.protected_assets.join(path),
			&state.Config.local_loading.protected_assets
		),
	};
	let is_local   =  match behavior {
		LoadingBehavior::Deny       => false,
		LoadingBehavior::Supplement => basedir.get_file(path).is_none(),
		LoadingBehavior::Override   => local_path.exists(),
	};
	if !(
			( is_local && local_path.exists())
		||	(!is_local && basedir.get_file(path).is_some())
	) {
		return Err((StatusCode::NOT_FOUND, ""));
	}
	let body = if is_local {
		let mut file   = File::open(local_path).await.ok().unwrap();
		let config     =  &state.Config.static_files;
		if file.metadata().await.unwrap().len() as usize > 1024 * config.stream_threshold {
			let reader = BufReader::with_capacity(1024 * config.read_buffer, file);
			let stream = ReaderStream::with_capacity(reader, 1024 * config.stream_buffer);
			Body::wrap_stream(stream)
		} else {
			let mut contents = vec![];
			file.read_to_end(&mut contents).await.unwrap();
			Body::from(contents)
		}
	} else {
		Body::from(basedir.get_file(path).unwrap().contents())
	};
	Ok(Response::builder()
		.status(StatusCode::OK)
		.header(
			header::CONTENT_TYPE,
			HeaderValue::from_str(mime_type.as_ref()).unwrap(),
		)
		.body(body)
		.unwrap()
	)
}

//		get_ping																
/// Availability check.
/// 
/// This endpoint is designed for use with uptime monitors. It simply returns
/// a 200 code and no content.
/// 
#[utoipa::path(
	get,
	path = "/api/ping",
	tag  = "health",
	responses(
		(status = 200, description = "Availability check")
	)
)]
pub async fn get_ping() {}

//		get_stats																
/// Produces various statistics about the service.
/// 
/// This endpoint returns a JSON object containing the following information:
/// 
///   - `started_at` - The date and time the application was started, in ISO
///                    8601 format.
///   - `uptime`     - The amount of time the application has been running, in
///                    seconds.
///   - `requests`   - The number of requests that have been handled since the
///                    application last started.
///   - `active`     - The number of current open connections.
///   - `codes`      - The counts of responses that have been handled, broken
///                    down by status code, since the application last started.
///   - `times`      - The average, maximum, and minimum response times, plus
///                    sample count, for the [configured periods](StatsOptions.stats_periods),
///                    and since the application last started.
///   - `endpoints`  - The counts of responses that have been handled, broken
///                    down by endpoint, since the application last started.
///   - `connections` - The average, maximum, and minimum open connections, plus
///                    sample count, for the [configured periods](StatsOptions.stats_periods),
///                    and since the application last started.
///   - `memory`     - The average, maximum, and minimum memory usage, plus
///                    sample count, for the [configured periods](StatsOptions.stats_periods),
///                    and since the application last started.
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
		(status = 200, description = "Application statistics", body = StatsResponse)
	)
)]
pub async fn get_stats(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
	//		Helper functions													
	fn initialize_map(
		periods: &HashMap<String, usize>,
		buffer:  &VecDeque<StatsForPeriod>,
	) -> IndexMap<String, StatsForPeriod> {
		let mut output: IndexMap<String, StatsForPeriod> = periods
			.iter()
			.sorted_by(|a, b| a.1.cmp(b.1))
			.map(|(name, _)| (name.clone(), StatsForPeriod { ..Default::default() }))
			.collect()
		;
		//	Loop through the circular buffer and calculate the stats
		for (i, stats) in buffer.iter().enumerate() {
			for (name, period) in periods.iter() {
				if i < *period {
					output.get_mut(name).unwrap().update(stats);
				}
			}
		}
		output
	}
	
	fn convert_map(
		input: IndexMap<String, StatsForPeriod>,
		all:   &StatsForPeriod
	) -> IndexMap<String, StatsResponseForPeriod> {
		let mut output: IndexMap<String, StatsResponseForPeriod> = input
			.into_iter()
			.map(|(key, value)| (key, StatsResponseForPeriod::from(&value)))
			.collect()
		;
		output.insert(s!("all"), StatsResponseForPeriod::from(all));
		output
	}
	
	//		Preparation															
	//	Lock source data
	let buffers      = state.Stats.buffers.read();
	
	//	Create pots for each period and process stats buffers
	let timing_input = initialize_map(&state.Config.stats_periods, &buffers.responses);
	let conn_input   = initialize_map(&state.Config.stats_periods, &buffers.connections);
	let memory_input = initialize_map(&state.Config.stats_periods, &buffers.memory);
	
	//	Unlock source data
	drop(buffers);
	
	//		Process stats														
	//	Lock source data
	let totals        = state.Stats.totals.lock();
	
	//	Convert the input stats data into the output stats data
	let timing_output = convert_map(timing_input, &totals.times);
	let conn_output   = convert_map(conn_input,   &totals.connections);
	let memory_output = convert_map(memory_input, &totals.memory);
	
	//		Build response data													
	let now        = Utc::now().naive_utc();
	let response   = Json(StatsResponse {
		started_at:  state.Stats.started_at,
		uptime:      (now - state.Stats.started_at).num_seconds() as u64,
		active:      state.Stats.connections.load(Ordering::Relaxed) as u64,
		requests:    state.Stats.requests.load(Ordering::Relaxed) as u64,
		codes:       totals.codes.clone(),
		times:       timing_output,
		endpoints:   HashMap::from_iter(
			totals.endpoints.clone()
				.into_iter()
				.map(|(key, value)| (key, StatsResponseForPeriod::from(&value)))
		),
		connections: conn_output,
		memory:      memory_output,
	});
	//	Unlock source data
	drop(totals);
	
	//		Response															
	response
}


