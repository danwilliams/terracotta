//! State for the assets-serving functionality.



//		Packages																										

use super::config::Config;
use include_dir::Dir;
use std::sync::Arc;



//		Traits																											

//§		StateProvider															
/// A trait for providing the application state aspects for assets.
pub trait StateProvider: Send + Sync + 'static {
	//		config																
	/// Gets the assets configuration.
	fn config(&self) -> &Config;
	
	//		assets_dir															
	/// The directory containing the static assets.
	fn assets_dir(&self) -> Arc<Dir<'static>>;
	
	//		content_dir															
	/// The directory containing the Markdown content.
	fn content_dir(&self) -> Arc<Dir<'static>>;
}


