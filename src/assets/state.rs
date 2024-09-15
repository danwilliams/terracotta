//! State for the assets-serving functionality.



//		Packages

use crate::assets::config::AssetsConfig;



//		Traits

//§		AssetsStateProvider														
/// A trait for providing the application state aspects for assets.
pub trait AssetsStateProvider: Send + Sync + 'static {
	//		assets_config														
	/// Gets the assets configuration.
	fn assets_config(&self) -> &AssetsConfig;
}


