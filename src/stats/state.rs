//! State for the statistics functionality.



//		Packages

use crate::stats::{
	config::StatsConfig,
	worker::AppStateStats,
};
use tokio::sync::RwLock;



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
	fn stats_state(&self) -> &RwLock<AppStateStats>;
}


