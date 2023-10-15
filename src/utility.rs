#![allow(non_snake_case)]

//		Packages

use crate::handlers;
use axum::http::{StatusCode, Uri, Method};
use chrono::{Duration, NaiveDateTime, Timelike, Utc};
use flume::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};
use ring::hmac;
use serde::{Deserialize, Serialize, Serializer};
use smart_default::SmartDefault;
use std::{
	collections::{BTreeMap, HashMap, VecDeque},
	net::IpAddr,
	path::PathBuf,
	sync::{Arc, atomic::AtomicUsize},
	thread::spawn,
};
use tera::Tera;
use url::form_urlencoded;
use utoipa::OpenApi;
use velcro::hash_map;



//		Enums

//		LoadingBehavior															
/// The possible options for loading local, non-baked-in resources.
#[derive(Debug, Deserialize, Serialize)]
pub enum LoadingBehavior {
	/// Deny loading of local resources.
	Deny,
	
	/// Load local resources if the baked-in resources are not present.
	Supplement,
	
	/// Load local resources if they exist, otherwise load baked-in resources.
	Override,
}



//		Structs

//		Config																	
/// The main configuration options for the application.
#[derive(Deserialize, Serialize, SmartDefault)]
pub struct Config {
	//		Public properties													
	/// The host to listen on.
	#[default(IpAddr::from([127, 0, 0, 1]))]
	pub host:          IpAddr,
	
	/// The port to listen on.
	#[default = 8000]
	pub port:          u16,
	
	/// The directory to store log files in.
	#[default = "log"]
	pub logdir:        String,
	
	/// The title of the application.
	#[default = "Terracotta"]
	pub title:         String,
	
	/// The loading behaviour for local, non-baked-in resources. This allows
	/// local resources to be used to complement the baked-in resources.
	pub local_loading: LocalLoading,
	
	/// The paths for local, non-baked-in resources.
	pub local_paths:   LocalPaths,
	
	/// The configuration options for serving static files.
	pub static_files:  StaticFiles,
	
	/// The configuration options for gathering and processing statistics.
	pub stats:         StatsOptions,
	
	/// The time periods to report statistics for. These will default to second,
	/// minute, hour, and day, and refer to the last such period of time from
	/// the current time, measured back from the start of the current second.
	/// They will be used to calculate the average, maximum, and minimum values
	/// for each period, and the number of values in each period. In addition,
	/// the statistics since the application started will always be reported.
	/// Note that any defaults specified here would be augmented by items added
	/// to config, and not replaced by them, so the desired periods NEED to be
	/// placed in the application config file. If omitted, there will be no
	/// registered periods.
	pub stats_periods: HashMap<String, usize>,
	
	/// A list of users and their passwords.
	#[default(HashMap::new())]
	pub users:         HashMap<String, String>,
}

//		LocalLoading															
/// The loading behaviour for local, non-baked-in resources.
#[derive(Deserialize, Serialize, SmartDefault)]
pub struct LocalLoading {
	//		Public properties													
	/// The loading behaviour for protected static assets.
	#[default(LoadingBehavior::Deny)]
	pub protected_assets: LoadingBehavior,
	
	/// The loading behaviour for public static assets.
	#[default(LoadingBehavior::Deny)]
	pub public_assets:    LoadingBehavior,
}

//		LocalPaths																
/// The local paths for non-baked-in resources.
#[derive(Deserialize, Serialize, SmartDefault)]
pub struct LocalPaths {
	//		Public properties													
	/// The path to the protected static assets.
	#[default = "content"]
	pub protected_assets: PathBuf,
	
	/// The path to the public static assets.
	#[default = "static"]
	pub public_assets:    PathBuf,
}

//		StaticFiles																
#[derive(Deserialize, Serialize, SmartDefault)]
/// The configuration options for serving static files.
pub struct StaticFiles {
	//		Public properties													
	/// The file size at which to start streaming, in KB. Below this size, the
	/// file will be read into memory and served all at once.
	#[default = 1000]
	pub stream_threshold: usize,
	
	/// The size of the stream buffer to use when streaming files, in KB.
	#[default = 256]
	pub stream_buffer:    usize,
	
	/// The size of the read buffer to use when streaming files, in KB.
	#[default = 128]
	pub read_buffer:      usize,
}

//		StatsOptions															
#[derive(Deserialize, Serialize, SmartDefault)]
/// The configuration options for gathering and processing statistics.
pub struct StatsOptions {
	//		Public properties													
	/// Whether to enable statistics gathering and processing. If enabled, there
	/// is a very small CPU overhead for each request, plus an
	/// individually-configurable amount of memory used to store the
	/// [response time buffer](StatsOptions.timing_buffer_size), the
	/// [connection count buffer](StatsOptions.connection_buffer_size), and the
	/// [memory usage buffer](StatsOptions.memory_buffer_size) (default 4.8MB
	/// per buffer). If disabled, the [statistics processing thread](start_stats_processor())
	/// will not be started, the buffers' capacities will not be reserved, and
	/// the [statistics middleware](crate::stats::stats_layer()) will do
	/// nothing. Under usual circumstances the statistics thread should easily
	/// be able to keep up with the incoming requests, even on a system with
	/// hundreds of CPU cores.
	#[default = true]
	pub enabled:                bool,
	
	/// The size of the buffer to use for storing response times, in seconds.
	/// Each entry (i.e. for one second) will take up 56 bytes, so the default
	/// of 86,400 seconds (one day) will take up around 4.8MB of memory. This
	/// seems like a reasonable default to be useful but not consume too much
	/// memory. Notably, the statistics output only looks at a maximum of the
	/// last day's-worth of data, so if a longer period than this is required
	/// the [`get_stats()`](handlers::get_stats()) code would need to be
	/// customised.
	#[default = 86_400]
	pub timing_buffer_size:     usize,
	
	/// The size of the buffer to use for storing connection data, in seconds.
	/// Each entry (i.e. for one second) will take up 56 bytes, so the default
	/// of 86,400 seconds (one day) will take up around 4.8MB of memory. This
	/// seems like a reasonable default to be useful but not consume too much
	/// memory. Notably, the statistics output only looks at a maximum of the
	/// last day's-worth of data, so if a longer period than this is required
	/// the [`get_stats()`](handlers::get_stats()) code would need to be
	/// customised.
	#[default = 86_400]
	pub connection_buffer_size: usize,
	
	/// The size of the buffer to use for storing memory usage data, in seconds.
	/// Each entry (i.e. for one second) will take up 56 bytes, so the default
	/// of 86,400 seconds (one day) will take up around 4.8MB of memory. This
	/// seems like a reasonable default to be useful but not consume too much
	/// memory. Notably, the statistics output only looks at a maximum of the
	/// last day's-worth of data, so if a longer period than this is required
	/// the [`get_stats()`](handlers::get_stats()) code would need to be
	/// customised.
	#[default = 86_400]
	pub memory_buffer_size:     usize,
}

//		AppState																
/// The application state.
/// 
/// This is used to store global state information that is shared between
/// requests.
/// 
#[allow(dead_code)]
pub struct AppState {
	//		Public properties													
	/// The application configuration.
	pub Config:   Config,
	
	/// The application statistics.
	pub Stats:    AppStats,
	
	/// The statistics queue that response times are added to. This is the
	/// sender side only. A queue is used so that each request-handling thread's
	/// stats middleware can send its metrics into the queue instead of updating
	/// a central, locked data structure. This avoids the need for locking and
	/// incineration routines, as the stats-handling thread can constantly
	/// process the queue and there will theoretically never be a large build-up
	/// of data in memory that has to be dealt with all at once.
	pub Queue:    Sender<ResponseMetrics>,
	
	/// The application secret.
	pub Secret:   [u8; 64],
	
	/// The HMAC key used to sign and verify sessions.
	pub Key:      hmac::Key,
	
	/// The Tera template engine.
	pub Template: Tera,
}

//		AppStats																
/// Various application statistics.
#[derive(SmartDefault)]
pub struct AppStats {
	//		Public properties													
	/// The date and time the application was started.
	pub started_at:  NaiveDateTime,
	
	/// The current number of open connections, i.e. requests that have not yet
	/// been responded to.
	pub connections: AtomicUsize,
	
	/// The number of requests that have been made. The number of responses will
	/// be incremented only when the request has been fully handled and a
	/// response generated.
	pub requests:    AtomicUsize,
	
	/// The average, maximum, minimum, and count for each area sampled. The data
	/// is wrapped inside a [`Mutex`] because it is important to update the
	/// count, use that exact count to calculate the average, and then store
	/// that average all in one atomic operation while blocking any other
	/// process from using the data. A [`parking_lot::Mutex`] is used instead of
	/// a [`std::sync::Mutex`] because it is theoretically faster in highly
	/// contended situations, but the main advantage is that it is infallible,
	/// and it does not have mutex poisoning.
	pub totals:      Mutex<AppStatsTotals>,
	
	/// Circular buffers of average, maximum, minimum, and count per second for
	/// each area sampled, for the individually-configured periods. The buffers
	/// are stored inside a [`RwLock`] because they are only ever written to a
	/// maximum of once per second. A [`parking_lot::RwLock`] is used instead of
	/// a [`std::sync::RwLock`] because it is theoretically faster in highly
	/// contended situations.
	pub buffers:     RwLock<AppStatsBuffers>,
}

//		AppStatsTotals															
/// The all-time application statistics totals for each area sampled.
#[derive(SmartDefault)]
pub struct AppStatsTotals {
	//		Public properties													
	/// The number of responses that have been handled, by status code.
	#[default(hash_map!{
		StatusCode::OK:                    0,
		StatusCode::UNAUTHORIZED:          0,
		StatusCode::NOT_FOUND:             0,
		StatusCode::INTERNAL_SERVER_ERROR: 0,
	})]
	pub codes:       HashMap<StatusCode, u64>,
	
	/// The average, maximum, and minimum response times since the application
	/// last started.
	pub times:       StatsForPeriod,
	
	/// The average, maximum, and minimum response times by endpoint since the
	/// application last started. These statistics are stored in a [`HashMap`]
	/// for ease.
	pub endpoints:   HashMap<Endpoint, StatsForPeriod>,
	
	/// The average, maximum, and minimum open connections by time period.
	pub connections: StatsForPeriod,
	
	/// The average, maximum, and minimum memory usage by time period.
	pub memory:      StatsForPeriod,
}

//		AppStatsBuffers															
/// Buffers for storing application statistics data.
#[derive(SmartDefault)]
pub struct AppStatsBuffers {
	//		Public properties													
	/// A circular buffer of response time stats per second for the configured
	/// period.
	pub responses:   VecDeque<StatsForPeriod>,
	
	/// A circular buffer of connection stats per second for the configured
	/// period.
	pub connections: VecDeque<StatsForPeriod>,
	
	/// A circular buffer of memory usage stats per second for the configured
	/// period.
	pub memory:      VecDeque<StatsForPeriod>,
}

//		StatsForPeriod															
/// Average, maximum, minimum, and count of values for a period of time.
#[derive(Clone, SmartDefault)]
pub struct StatsForPeriod {
	//		Public properties													
	/// The date and time the period started.
	#[default(Utc::now().naive_utc())]
	pub started_at: NaiveDateTime,
	
	/// Average value.
	pub average:    f64,
	
	/// Maximum value.
	pub maximum:    u64,
	
	/// Minimum value.
	pub minimum:    u64,
	
	/// The total number of values.
	pub count:      u64,
}

impl StatsForPeriod {
	//		update																
	/// Updates the stats with new data.
	/// 
	/// This function will compare the new data with the existing data and
	/// update the stats accordingly. The maximum and minimum values will be
	/// updated if the new data is higher or lower than the existing values,
	/// and the count will be the combined count of the existing and new data.
	/// 
	/// Of particular note is the treatment of the average value. This is
	/// calculated using a weighted average, combining the existing and new
	/// averages using the count of each set of data as a weighting factor.
	/// This means that the average value will be closer to the average of the
	/// new data if the existing data is much larger than the new data, and vice
	/// versa.
	/// 
	/// The start time will not be updated.
	/// 
	/// # Parameters
	/// 
	/// * `stats` - The stats to update with.
	/// 
	pub fn update(&mut self, stats: &StatsForPeriod) {
		if (stats.minimum < self.minimum && stats.count > 0) || self.count == 0 {
			self.minimum = stats.minimum;
		}
		if stats.maximum > self.maximum {
			self.maximum = stats.maximum;
		}
		self.count      += stats.count;
		if self.count > 0  && stats.count > 0 {
			let weight   = stats.count as f64 / self.count as f64;
			self.average = self.average * (1.0 - weight) + stats.average * weight;
		}
	}
}

//		ResponseMetrics															
/// Metrics for a single response.
/// 
/// This is used by the statistics queue in [`AppState.Queue`].
/// 
#[derive(SmartDefault)]
pub struct ResponseMetrics {
	//		Public properties													
	/// The endpoint that was requested.
	pub endpoint:    Endpoint,
	
	/// The date and time the request started.
	#[default(Utc::now().naive_utc())]
	pub started_at:  NaiveDateTime,
	
	/// The time the response took to be generated, in microseconds.
	pub time_taken:  u64,
	
	/// The status code of the response.
	pub status_code: StatusCode,
	
	/// The number of open connections at the time the response was generated.
	pub connections: u64,
	
	/// The amount of memory allocated at the time the response was generated,
	/// in bytes.
	pub memory:      u64,
}

//		Endpoint																
/// A formalised definition of an endpoint for identification.
#[derive(Clone, Eq, Hash, PartialEq, SmartDefault)]
pub struct Endpoint {
	//		Public properties													
	/// The path of the endpoint, minus any query parameters. As this is just
	/// the path, it does not contain scheme or authority (host), and hence is
	/// not a full URI.
	pub path:   String,
	
	/// The HTTP verb of the endpoint.
	pub method: Method,
}

impl Serialize for Endpoint {
	//		serialize															
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&format!("{} {}", self.method, self.path))
	}
}

//		ApiDoc																	
/// The OpenAPI documentation for the API.
#[derive(OpenApi)]
#[openapi(
	paths(
		handlers::get_ping,
		handlers::get_stats,
	),
	components(
		schemas(
			handlers::StatsResponse,
			handlers::StatsResponseForPeriod,
		),
	),
	tags(
		(name = "health", description = "Health check endpoints"),
	)
)]
pub struct ApiDoc;



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
pub fn extract_uri_query_parts(uri: Uri) -> HashMap<String, String> {
	uri
		.query()
		.map(|v| {
			form_urlencoded::parse(v.as_bytes())
				.into_owned()
				.collect()
		})
		.unwrap_or_else(HashMap::new)
}

//		build_uri																
/// Builds a URI from a path and a set of query parameters.
/// 
/// # Parameters
/// 
/// * `path`   - The path to build the URI from.
/// * `params` - The query parameters to add to the URI.
/// 
pub fn build_uri(path: String, params: HashMap<String, String>) -> Uri {
	Uri::builder()
		.path_and_query(format!("{}?{}",
			path,
			params
				.iter()
				.map(|(k, v)| format!("{}={}", k, v))
				.collect::<Vec<String>>()
				.join("&")
		))
		.build()
		.unwrap()
}

//		serialize_status_codes													
/// Returns a list of serialised status code entries and their values.
/// 
/// This function is used by [`serde`] to serialise a list of status codes and
/// their associated values. It returns the list sorted by status code.
/// 
/// # Parameters
/// 
/// * `status_codes` - The status codes to serialise, as keys, against values.
/// * `serializer`   - The serialiser to use.
/// 
pub fn serialize_status_codes<S>(status_codes: &HashMap<StatusCode, u64>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	let codes: BTreeMap<String, u64> = status_codes
		.iter()
		.map(|(key, value)| (key.to_string(), *value))
		.collect()
	;
	codes.serialize(serializer)
}

//		start_stats_processor													
/// Starts the statistics processor.
/// 
/// This function starts a thread that will process the statistics queue in
/// [`AppState.Queue`]. It will run until the channel is disconnected.
/// 
/// The processing of the statistics is done in a separate thread so that the
/// request-handling threads can continue to handle requests without being
/// blocked by the statistics processing. This way, none of them are ever
/// affected more than others. The stats-handling thread blocks on the queue, so
/// it will only process a response time when one is available.
/// 
/// # Parameters
/// 
/// * `receiver` - The receiving end of the queue.
/// * `appstate` - The application state.
/// 
pub fn start_stats_processor(receiver: Receiver<ResponseMetrics>, appstate: Arc<AppState>) {
	//	Fixed time period of the current second
	let mut current_second = Utc::now().naive_utc().with_nanosecond(0).unwrap();
	//	Cumulative stats for the current second
	let mut timing_stats   = StatsForPeriod::default();
	let mut conn_stats     = StatsForPeriod::default();
	let mut memory_stats   = StatsForPeriod::default();
	
	//	Initialise circular buffers. We reserve the capacities here right at the
	//	start so that the application always uses exactly the same amount of
	//	memory for the buffers, so that any memory-usage issues will be spotted
	//	immediately. For instance, if someone set the config value high enough
	//	to store a year's worth of data (around 1.8GB) and the system didn't
	//	have enough memory it would fail right away, instead of gradually
	//	building up to that point which would make it harder to diagnose.
	{
		let mut buffers    = appstate.Stats.buffers.write();
		buffers.responses  .reserve(appstate.Config.stats.timing_buffer_size);
		buffers.connections.reserve(appstate.Config.stats.connection_buffer_size);
		buffers.memory     .reserve(appstate.Config.stats.memory_buffer_size);
	}
	
	//	Queue processing loop
	spawn(move || loop {
		//	Wait for message - this is a blocking call
		match receiver.recv() {
			Ok(response_time) => {
				//	Process response time
				(timing_stats, conn_stats, memory_stats, current_second) = stats_processor(
					Arc::clone(&appstate), response_time, timing_stats, conn_stats, memory_stats, current_second
				);
			},
			Err(_)            => {
				eprintln!("Channel has been disconnected, exiting thread.");
				break;
			},
		}
	});
}

//		stats_processor															
/// Processes a single response time.
/// 
/// This function processes a single response metrics sample, updating the
/// calculated statistics accordingly.
/// 
/// # Parameters
/// 
/// * `appstate`       - The application state.
/// * `metrics`        - The response metrics to process, received from the
///                      statistics queue in [`AppState.Queue`].
/// * `timing_stats`   - The cumulative timing stats for the current second.
/// * `conn_stats`     - The cumulative connection stats for the current second.
/// * `memory_stats`   - The cumulative memory stats for the current second.
/// * `current_second` - The current second.
/// 
fn stats_processor(
	appstate:           Arc<AppState>,
	metrics:            ResponseMetrics,
	mut timing_stats:   StatsForPeriod,
	mut conn_stats:     StatsForPeriod,
	mut memory_stats:   StatsForPeriod,
	mut current_second: NaiveDateTime
) -> (StatsForPeriod, StatsForPeriod, StatsForPeriod, NaiveDateTime) {
	//		Preparation															
	//	Prepare new stats
	let newstats = StatsForPeriod {
		average:   metrics.time_taken as f64,
		maximum:   metrics.time_taken,
		minimum:   metrics.time_taken,
		count:     1,
		..Default::default()
	};
	let constats = StatsForPeriod {
		average:   metrics.connections as f64,
		maximum:   metrics.connections,
		minimum:   metrics.connections,
		count:     1,
		..Default::default()
	};
	let memstats = StatsForPeriod {
		average:   metrics.memory as f64,
		maximum:   metrics.memory,
		minimum:   metrics.memory,
		count:     1,
		..Default::default()
	};
	
	//	Increment cumulative stats
	timing_stats.update(&newstats);
	conn_stats.update(&constats);
	memory_stats.update(&memstats);
	
	//		Update statistics													
	//	Lock source data
	let mut totals = appstate.Stats.totals.lock();
	
	//	Update responses counter
	*totals.codes.entry(metrics.status_code).or_insert(0) += 1;
	
	//	Update response time stats
	totals.times.update(&newstats);
	
	//	Update endpoint response time stats
	totals.endpoints
		.entry(metrics.endpoint)
		.and_modify(|ep_stats| ep_stats.update(&newstats))
		.or_insert(newstats)
	;
	
	//	Update connections usage stats
	totals.connections.update(&constats);
	
	//	Update memory usage stats
	totals.memory.update(&memstats);
	
	//	Unlock source data
	drop(totals);
	
	//		Check time period													
	let new_second      = metrics.started_at.with_nanosecond(0).unwrap();
	
	//	Check to see if we've moved into a new time period. We want to increment
	//	the request count and total response time until it "ticks" over into
	//	another second. At this point it will calculate an average and add this
	//	data (average, min, max) to a fixed-length circular buffer of seconds.
	//	This way, the last period's data can be calculated by looking through
	//	the circular buffer of seconds.
	if new_second > current_second {
		let elapsed     = (new_second - current_second).num_seconds();
		let mut buffers = appstate.Stats.buffers.write();
		//	Timing stats buffer
		for i in 0..elapsed {
			if buffers.responses.len() == appstate.Config.stats.timing_buffer_size {
				buffers.responses.pop_back();
			}
			timing_stats.started_at = current_second + Duration::seconds(i);
			buffers.responses.push_front(timing_stats);
			timing_stats            = StatsForPeriod::default();
		}
		//	Connections stats buffer
		for i in 0..elapsed {
			if buffers.connections.len() == appstate.Config.stats.connection_buffer_size {
				buffers.connections.pop_back();
			}
			conn_stats.started_at   = current_second + Duration::seconds(i);
			buffers.connections.push_front(conn_stats);
			conn_stats              = StatsForPeriod::default();
		}
		//	Memory stats buffer
		for i in 0..elapsed {
			if buffers.memory.len() == appstate.Config.stats.memory_buffer_size {
				buffers.memory.pop_back();
			}
			memory_stats.started_at = current_second + Duration::seconds(i);
			buffers.memory.push_front(memory_stats);
			memory_stats            = StatsForPeriod::default();
		}
		current_second = new_second;
	}
	
	(timing_stats, conn_stats, memory_stats, current_second)
}


