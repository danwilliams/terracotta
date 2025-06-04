#![allow(clippy::exhaustive_structs, reason = "Configuration structs")]

//! Configuration for the statistics functionality.



//		Packages																										

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::collections::HashMap;



//		Structs																											

//		Config																	
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
/// The configuration options for gathering and processing statistics.
pub struct Config {
	//		Public properties													
	/// Whether to enable statistics gathering and processing. If enabled, there
	/// is a very small CPU overhead for each request, plus an
	/// individually-configurable amount of memory used to store the
	/// [response time buffer](Config#structfield.timing_buffer_size), the
	/// [connection count buffer](Config#structfield.connection_buffer_size),
	/// and the [memory usage buffer](Config#structfield.memory_buffer_size)
	/// (default 4.8MB per buffer). If disabled, the
	/// [statistics processing thread](crate::stats::worker::start()) will not
	/// be started, the buffers' capacities will not be reserved, and the
	/// [statistics middleware](crate::stats::middleware::stats_layer()) will do
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
	/// the [`get_stats()`](crate::stats::handlers::get_stats()) code would need
	/// to be customised.
	#[default = 86_400]
	pub timing_buffer_size:     usize,
	
	/// The size of the buffer to use for storing connection data, in seconds.
	/// Each entry (i.e. for one second) will take up 56 bytes, so the default
	/// of 86,400 seconds (one day) will take up around 4.8MB of memory. This
	/// seems like a reasonable default to be useful but not consume too much
	/// memory. Notably, the statistics output only looks at a maximum of the
	/// last day's-worth of data, so if a longer period than this is required
	/// the [`get_stats()`](crate::stats::handlers::get_stats()) code would need
	/// to be customised.
	#[default = 86_400]
	pub connection_buffer_size: usize,
	
	/// The size of the buffer to use for storing memory usage data, in seconds.
	/// Each entry (i.e. for one second) will take up 56 bytes, so the default
	/// of 86,400 seconds (one day) will take up around 4.8MB of memory. This
	/// seems like a reasonable default to be useful but not consume too much
	/// memory. Notably, the statistics output only looks at a maximum of the
	/// last day's-worth of data, so if a longer period than this is required
	/// the [`get_stats()`](crate::stats::handlers::get_stats()) code would need
	/// to be customised.
	#[default = 86_400]
	pub memory_buffer_size:     usize,
	
	/// The interval at which to send ping messages to WebSocket clients, in
	/// seconds. This is used to check the connection is still alive.
	#[default = 60]
	pub ws_ping_interval:       usize,
	
	/// The timeout for WebSocket ping messages, in seconds. If a pong message
	/// is not received in reply to the outgoing ping message within this time,
	/// the connection will be closed.
	#[default = 10]
	pub ws_ping_timeout:        usize,
	
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
	pub periods:                HashMap<String, usize>,
}


