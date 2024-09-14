//! Configuration for the application.



//		Packages

use core::net::IpAddr;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::{
	collections::HashMap,
	path::PathBuf,
};



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
	#[default = 1_000]
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
	/// per buffer). If disabled, the [statistics processing thread](crate::stats::middleware::start_stats_processor())
	/// will not be started, the buffers' capacities will not be reserved, and
	/// the [statistics middleware](crate::stats::middleware::stats_layer())
	/// will do nothing. Under usual circumstances the statistics thread should
	/// easily be able to keep up with the incoming requests, even on a system
	/// with hundreds of CPU cores.
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
}


