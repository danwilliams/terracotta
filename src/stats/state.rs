//! State for the statistics functionality.



//		Packages

use crate::stats::{
	config::StatsConfig,
	worker::AppStateStats,
};



//		Traits

//§		StatsStateProvider														
/// A trait for providing the application state aspects for statistics.
pub trait StatsStateProvider: Send + Sync + 'static {
	//		stats_config														
	/// Gets the statistics configuration.
	fn stats_config(&self) -> &StatsConfig;
	
	//		stats_state															
	/// Gets the statistics state.
	fn stats_state(&self) -> &AppStateStats;
}


