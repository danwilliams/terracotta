//! Statistics middleware.



//		Packages

use super::state::StateProvider;
use axum::http::{Method, StatusCode};
use chrono::{Duration, NaiveDateTime, Timelike, Utc};
use serde::{Serialize, Serializer};
use smart_default::SmartDefault;
use std::{
	collections::VecDeque,
	sync::Arc,
};
use tokio::{
	select,
	spawn,
	sync::broadcast,
	time::{interval, sleep},
};
use tracing::error;
use utoipa::ToSchema;



//		Structs

//		Endpoint																
/// A formalised definition of an endpoint for identification.
#[derive(Clone, Debug, Eq, Hash, PartialEq, SmartDefault)]
pub struct Endpoint {
	//		Public properties													
	/// The path of the endpoint, minus any query parameters. As this is just
	/// the path, it does not contain scheme or authority (host), and hence is
	/// not a full URI.
	pub path:   String,
	
	/// The HTTP verb of the endpoint.
	pub method: Method,
}

//󰭅		Serialize																
impl Serialize for Endpoint {
	//		serialize															
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&format!("{} {}", self.method, self.path))
	}
}

//		StatsForPeriod															
/// Average, maximum, minimum, and count of values for a period of time.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, SmartDefault)]
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

//󰭅		StatsForPeriod															
impl StatsForPeriod {
	//		initialize															
	/// Initialises the stats based on a single starting value.
	/// 
	/// # Parameters
	/// 
	/// * `value` - The single value to start with. This will be applied to the
	///             average, maximum, and minimum values, and the count will be
	///             set to 1.
	/// 
	pub fn initialize(value: u64) -> Self {
		#[expect(clippy::cast_precision_loss, reason = "Not expected to get anywhere near 52 bits")]
		Self {
			average: value as f64,
			maximum: value,
			minimum: value,
			count:   1,
			..Default::default()
		}
	}
	
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
	pub fn update(&mut self, stats: &Self) {
		if (stats.minimum < self.minimum && stats.count > 0) || self.count == 0 {
			self.minimum = stats.minimum;
		}
		if stats.maximum > self.maximum {
			self.maximum = stats.maximum;
		}
		self.count       = self.count.saturating_add(stats.count);
		if self.count > 0  && stats.count > 0 {
			#[expect(clippy::cast_precision_loss, reason = "Not expected to get anywhere near 52 bits")]
			let weight   = stats.count as f64 / self.count as f64;
			self.average = self.average.mul_add(1.0 - weight, stats.average * weight);
		}
	}
}

//		AllStatsForPeriod														
/// Average, maximum, minimum, and count of values for a period of time, for all
/// areas being measured.
#[derive(Clone, Debug, Default, PartialEq, Serialize, ToSchema)]
pub struct AllStatsForPeriod {
	//		Public properties													
	/// The average, maximum, and minimum response times in microseconds, plus
	/// sample count, for the most recent second.
	pub times:       StatsForPeriod,
	
	/// The average, maximum, and minimum open connections, plus sample count,
	/// for the most recent second.
	pub connections: StatsForPeriod,
	
	/// The average, maximum, and minimum memory usage in bytes, plus sample
	/// count, for the most recent second.
	pub memory:      StatsForPeriod,
}

//		ResponseMetrics															
/// Metrics for a single response.
/// 
/// This is used by the statistics queue in [`AppState.stats.Queue`].
/// 
#[derive(Clone, Debug, Eq, PartialEq, SmartDefault)]
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



//		Functions

//		start_stats_processor													
/// Starts the statistics processor.
/// 
/// This function starts a thread that will process the statistics queue in
/// [`AppState.stats.Queue`]. It will run until the channel is disconnected.
/// 
/// The processing of the statistics is done in a separate thread so that the
/// request-handling threads can continue to handle requests without being
/// blocked by the statistics processing. This way, none of them are ever
/// affected more than others. The stats-handling thread blocks on the queue, so
/// it will only process a response time when one is available.
/// 
/// The thread will also wake up every second to ensure that the period that has 
/// just ended gets wrapped up. This is necessary because the thread otherwise
/// only wakes up when the queue has data in it, and if there is a period of
/// inactivity then the current period will not be completed until the next
/// request comes in. This can lead to a long delay until the statistics are
/// updated, which is undesirable because the buffer will be stuck at the
/// position of the last period to be completed.
/// 
/// Although this periodic wake-up does incur a very slight overhead, it is
/// extremely small, and ensures that the statistics are always up-to-date.
/// 
/// # Parameters
/// 
/// * `receiver`     - The receiving end of the queue.
/// * `shared_state` - The shared application state.
/// 
pub async fn start_stats_processor<SP: StateProvider>(shared_state: &Arc<SP>) {
	if !shared_state.stats_config().enabled {
		return;
	}
	let appstate            = Arc::clone(shared_state);
	let (sender, receiver)  = flume::unbounded();
	let (tx, rx)            = broadcast::channel(10);
	let mut stats_state     = appstate.stats_state().write().await;
	stats_state.queue       = Some(sender);
	stats_state.broadcaster = Some(tx);
	stats_state.listener    = Some(rx);
	//	Fixed time period of the current second
	let mut current_second  = Utc::now().naive_utc().with_nanosecond(0).unwrap();
	//	Cumulative stats for the current second
	let mut timing_stats    = StatsForPeriod::default();
	let mut conn_stats      = StatsForPeriod::default();
	let mut memory_stats    = StatsForPeriod::default();
	
	//	Initialise circular buffers. We reserve the capacities here right at the
	//	start so that the application always uses exactly the same amount of
	//	memory for the buffers, so that any memory-usage issues will be spotted
	//	immediately. For instance, if someone set the config value high enough
	//	to store a year's worth of data (around 1.8GB) and the system didn't
	//	have enough memory it would fail right away, instead of gradually
	//	building up to that point which would make it harder to diagnose.
	{
		let mut buffers = stats_state.data.buffers.write();
		buffers.responses  .reserve(appstate.stats_config().timing_buffer_size);
		buffers.connections.reserve(appstate.stats_config().connection_buffer_size);
		buffers.memory     .reserve(appstate.stats_config().memory_buffer_size);
	}
	drop(stats_state);
	
	//	Wait until the start of the next second, to align with it so that the
	//	tick interval change happens right after the second change, to wrap up
	//	the data for the period that has just ended.
	#[expect(clippy::arithmetic_side_effects, reason = "Nothing interesting can happen here")]
	sleep((current_second + Duration::seconds(1) - Utc::now().naive_utc()).to_std().unwrap()).await;
	
	//	Queue processing loop
	let mut timer = interval(Duration::seconds(1).to_std().unwrap());
	drop(spawn(async move { loop { select!{
		_ = timer.tick() => {
			//	Ensure last period is wrapped up
			stats_processor(
				&appstate,
				None,
				&mut timing_stats,
				&mut conn_stats,
				&mut memory_stats,
				&mut current_second,
			).await;
		}
		//	Wait for message - this is a blocking call
		message = receiver.recv_async() => {
			if let Ok(response_time) = message {
				//	Process response time
				stats_processor(
					&appstate,
					Some(response_time),
					&mut timing_stats,
					&mut conn_stats,
					&mut memory_stats,
					&mut current_second,
				).await;
			} else {
				error!("Channel has been disconnected, exiting thread.");
				break;
			}
		}
	}}}));
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
///                      statistics queue in [`AppState.stats.Queue`]. If
///                      [`None`], then no stats will be added or altered, and
///                      no counters will be incremented, but the most-recent
///                      period will be checked and wrapped up if not already
///                      done.
/// * `timing_stats`   - The cumulative timing stats for the current second.
/// * `conn_stats`     - The cumulative connection stats for the current second.
/// * `memory_stats`   - The cumulative memory stats for the current second.
/// * `current_second` - The current second.
/// 
async fn stats_processor<SP: StateProvider>(
	appstate:       &Arc<SP>,
	metrics:        Option<ResponseMetrics>,
	timing_stats:   &mut StatsForPeriod,
	conn_stats:     &mut StatsForPeriod,
	memory_stats:   &mut StatsForPeriod,
	current_second: &mut NaiveDateTime
) {
	//		Helper functions													
	/// Updates a buffer with new data.
	fn update_buffer(
		buffer:             &mut VecDeque<StatsForPeriod>,
		buffer_size:        usize,
		stats:              &mut StatsForPeriod,
		current_second:     &NaiveDateTime,
		elapsed:            i64,
		message:            &mut AllStatsForPeriod,
		mut update_message: impl FnMut(&mut StatsForPeriod, &mut AllStatsForPeriod),
	) {
		for i in 0..elapsed {
			if buffer.len() == buffer_size {
				_ = buffer.pop_back();
			}
			stats.started_at = current_second.checked_add_signed(Duration::seconds(i)).unwrap_or(*current_second);
			buffer.push_front(*stats);
			update_message(stats, message);
			*stats           = StatsForPeriod::default();
		}
	}
	
	//		Preparation															
	let new_second: NaiveDateTime;
	#[expect(clippy::shadow_reuse, reason = "Clear purpose")]
	if let Some(metrics) = metrics {
		//	Prepare new stats
		let new_timing_stats = StatsForPeriod::initialize(metrics.time_taken);
		let new_conn_stats   = StatsForPeriod::initialize(metrics.connections);
		let new_memory_stats = StatsForPeriod::initialize(metrics.memory);
		
		//	Increment cumulative stats
		timing_stats.update(&new_timing_stats);
		conn_stats  .update(&new_conn_stats);
		memory_stats.update(&new_memory_stats);
		
	//		Update statistics													
		//	Lock source data
		let stats_state = appstate.stats_state().read().await;
		let mut totals = stats_state.data.totals.lock();
		
		//	Update responses counter
		_ = totals.codes.entry(metrics.status_code).and_modify(|e| *e = e.saturating_add(1)).or_insert(1);
		
		//	Update response time stats
		totals.times.update(&new_timing_stats);
		
		//	Update endpoint response time stats
		_ = totals.endpoints
			.entry(metrics.endpoint)
			.and_modify(|ep_stats| ep_stats.update(&new_timing_stats))
			.or_insert(new_timing_stats)
		;
		
		//	Update connections usage stats
		totals.connections.update(&new_conn_stats);
		
		//	Update memory usage stats
		totals.memory.update(&new_memory_stats);
		
		//	Unlock source data
		drop(totals);
		drop(stats_state);
		
	//		Check time period													
		new_second = metrics.started_at.with_nanosecond(0).unwrap();
	} else {
		new_second = Utc::now().naive_utc().with_nanosecond(0).unwrap();
	};
	
	//	Check to see if we've moved into a new time period. We want to increment
	//	the request count and total response time until it "ticks" over into
	//	another second. At this point it will calculate an average and add this
	//	data (average, min, max) to a fixed-length circular buffer of seconds.
	//	This way, the last period's data can be calculated by looking through
	//	the circular buffer of seconds.
	if new_second > *current_second {
		#[expect(clippy::arithmetic_side_effects, reason = "Nothing interesting can happen here")]
		let elapsed     = (new_second - *current_second).num_seconds();
		let stats_state = appstate.stats_state().read().await;
		let mut buffers = stats_state.data.buffers.write();
		let mut message = AllStatsForPeriod::default();
		//	Timing stats buffer
		update_buffer(
			&mut buffers.responses,
			appstate.stats_config().timing_buffer_size,
			timing_stats,
			current_second,
			elapsed,
			&mut message,
			|stats, msg| { msg.times = *stats; },
		);
		//	Connections stats buffer
		update_buffer(
			&mut buffers.connections,
			appstate.stats_config().connection_buffer_size,
			conn_stats,
			current_second,
			elapsed,
			&mut message,
			|stats, msg| { msg.connections = *stats; },
		);
		//	Memory stats buffer
		update_buffer(
			&mut buffers.memory,
			appstate.stats_config().memory_buffer_size,
			memory_stats,
			current_second,
			elapsed,
			&mut message,
			|stats, msg| { msg.memory = *stats; },
		);
		drop(buffers);
		*stats_state.data.last_second.write() = *current_second;
		*current_second = new_second;
		if let Some(ref broadcaster) = stats_state.broadcaster {
			drop(broadcaster.send(message).inspect_err(|err| error!("Failed to broadcast stats: {err}")));
		}
		drop(stats_state);
	}
}


