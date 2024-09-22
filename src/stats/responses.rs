//! Response data for statistics functionality.



//		Packages

use super::{
	worker::{Endpoint, StatsForPeriod},
	utility::serialize_status_codes,
};
use axum::http::StatusCode;
use chrono::NaiveDateTime;
use indexmap::IndexMap;
use serde::Serialize;
use std::collections::HashMap;

#[cfg(feature = "utoipa")]
use utoipa::ToSchema;



//		Structs

//		StatsResponse															
/// The application statistics returned by the `/api/stats` endpoint.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[non_exhaustive]
pub struct StatsResponse {
	//		Public properties													
	/// The date and time the application was started.
	pub started_at:  NaiveDateTime,
	
	/// The latest second period that has been completed.
	pub last_second: NaiveDateTime,
	
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

//		StatsHistoryResponse													
/// The application statistics returned by the `/api/stats/history` endpoint.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[non_exhaustive]
pub struct StatsHistoryResponse {
	//		Public properties													
	/// The latest second period that has been completed.
	pub last_second: NaiveDateTime,
	
	/// The average, maximum, and minimum response times in microseconds, plus
	/// sample count, per second for every second since the application last
	/// started, or up until the end of the [configured buffer](super::config::Config#structfield.timing_buffer_size).
	pub times:       Vec<StatsResponseForPeriod>,
	
	/// The average, maximum, and minimum open connections, plus sample count,
	/// per second for every second since the application last started, or up
	/// until the end of the [configured buffer](super::config::Config#structfield.connection_buffer_size).
	pub connections: Vec<StatsResponseForPeriod>,
	
	/// The average, maximum, and minimum memory usage in bytes, plus sample
	/// count, per second for every second since the application last started,
	/// or up until the end of the [configured buffer](super::config::Config#structfield.memory_buffer_size).
	pub memory:      Vec<StatsResponseForPeriod>,
}

//		StatsResponseForPeriod													
/// Average, maximum, minimum, and count of values for a period of time.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[non_exhaustive]
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

//󰭅		From &StatsForPeriod													
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


