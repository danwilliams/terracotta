#![allow(clippy::exhaustive_structs, reason = "Configuration structs")]

//! Configuration for the application.



//		Packages

#[cfg(any(feature = "assets", feature = "tera"))]
use serde::{Deserialize, Serialize};

#[cfg(feature = "tera")]
use ::{
	smart_default::SmartDefault,
	std::path::PathBuf,
};



//		Enums

//		LoadingBehavior															
/// The possible options for loading local, non-baked-in resources.
#[cfg(any(feature = "assets", feature = "tera"))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[expect(clippy::exhaustive_enums, reason = "Exhaustive")]
pub enum LoadingBehavior {
	/// Deny loading of local resources.
	Deny,
	
	/// Load local resources if the baked-in resources are not present.
	Supplement,
	
	/// Load local resources if they exist, otherwise load baked-in resources.
	Override,
}



//		Structs

//		HtmlTemplates															
/// Loading configuration for HTML templates.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
#[cfg(feature = "tera")]
pub struct HtmlTemplates {
	//		Public properties													
	/// The loading behaviour for local, non-baked-in HTML templates. This
	/// allows local HTML templates to be used to complement the baked-in
	/// templates.
	#[default(LoadingBehavior::Deny)]
	pub behavior:   LoadingBehavior,
	
	/// The path to the local, non-baked-in HTML templates.
	#[default = "html"]
	pub local_path: PathBuf,
}


