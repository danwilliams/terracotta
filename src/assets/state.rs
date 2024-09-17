//! State for the assets-serving functionality.



//		Packages

use super::config::AssetsConfig;
use include_dir::Dir;
use std::sync::Arc;



//		Traits

//§		AssetsStateProvider														
/// A trait for providing the application state aspects for assets.
pub trait AssetsStateProvider: Send + Sync + 'static {
	//		assets_config														
	/// Gets the assets configuration.
	fn assets_config(&self) -> &AssetsConfig;
	
	//		assets_dir															
	/// The directory containing the static assets.
	fn assets_dir(&self) -> Arc<Dir<'static>>;
	
	//		content_dir															
	/// The directory containing the Markdown content.
	fn content_dir(&self) -> Arc<Dir<'static>>;
}


