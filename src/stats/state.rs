//! State for the statistics functionality.



//		Packages

use super::{
	config::StatsConfig,
	worker::{AllStatsForPeriod, Endpoint, ResponseMetrics, StatsForPeriod},
};
use axum::http::StatusCode;
use chrono::{NaiveDateTime, Utc};
use core::sync::atomic::AtomicUsize;
use flume::Sender;
use parking_lot::{Mutex, RwLock};
use smart_default::SmartDefault;
use std::collections::{HashMap, VecDeque};
use tokio::{
	sync::{
		RwLock as AsyncRwLock,
		broadcast::Sender as Broadcaster,
	},
};
use velcro::hash_map;



//		Structs

//		AppStateStats															
/// Statistics-related central constructs to be stored in application state.
/// 
/// This is used to store global state information that is shared between
/// requests, specific to what is used for statistics purposes.
/// 
#[derive(SmartDefault)]
pub struct AppStateStats {
	//		Public properties													
	/// The application statistics data.
	pub data:      AppStats,
	
	/// The statistics queue that response times are added to. This is the
	/// sender side only. A queue is used so that each request-handling thread's
	/// stats middleware can send its metrics into the queue instead of updating
	/// a central, locked data structure. This avoids the need for locking and
	/// incineration routines, as the stats-handling thread can constantly
	/// process the queue and there will theoretically never be a large build-up
	/// of data in memory that has to be dealt with all at once.
	pub queue:     Option<Sender<ResponseMetrics>>,
	
	/// The statistics broadcast channel that period-based statistics are added
	/// to. This is the receiver side only. Each interested party can subscribe
	/// to this channel to receive the latest statistics for a given period on
	/// a real-time basis.
	pub broadcast: Option<Broadcaster<AllStatsForPeriod>>,
}

//		AppStats																
/// Various application statistics.
#[derive(SmartDefault)]
pub struct AppStats {
	//		Public properties													
	/// The date and time the application was started.
	#[default(Utc::now().naive_utc())]
	pub started_at:  NaiveDateTime,
	
	/// The latest second period that has been completed.
	pub last_second: RwLock<NaiveDateTime>,
	
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



//		Traits

//§		StatsStateProvider														
/// A trait for providing the application state aspects for statistics.
pub trait StatsStateProvider: Send + Sync + 'static {
	//		stats_config														
	/// Gets the statistics configuration.
	fn stats_config(&self) -> &StatsConfig;
	
	//		stats_state															
	/// Gets the statistics state.
	/// 
	/// Notably, this is behind a read-write lock, so that the broadcaster and
	/// queue can be set when the stats processor starts. From that point on,
	/// all stats-processing access is read-only, which means no delay in
	/// obtaining a lock, and all the internal locks are kept in place in order
	/// to allow specific access.
	/// 
	fn stats_state(&self) -> &AsyncRwLock<AppStateStats>;
}


