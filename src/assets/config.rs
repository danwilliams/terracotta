#![allow(clippy::exhaustive_structs, reason = "Configuration structs")]

//! Configuration for the assets-serving functionality.



//		Packages

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::path::PathBuf;



//		Enums

//		LoadingBehavior															
/// The possible options for loading local, non-baked-in resources.
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

//		Config																	
/// The configuration options for gathering and processing statistics.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
pub struct Config {
	//		Public properties													
	/// Loading configuration for HTML templates.
	pub html_templates:   HtmlTemplates,
	
	/// Loading configuration for protected static assets.
	pub protected_assets: ProtectedAssets,
	
	/// Loading configuration for public static assets.
	pub public_assets:    PublicAssets,
	
	/// The configuration options for serving static files.
	pub static_files:     StaticFiles,
}

//		HtmlTemplates															
/// Loading configuration for HTML templates.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
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

//		ProtectedAssets															
/// Loading configuration for protected static assets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
pub struct ProtectedAssets {
	//		Public properties													
	/// The loading behaviour for local, non-baked-in protected static assets.
	/// This allows local assets to be used to complement the baked-in assets.
	#[default(LoadingBehavior::Deny)]
	pub behavior:   LoadingBehavior,
	
	/// The path to the local, non-baked-in protected static assets.
	#[default = "content"]
	pub local_path: PathBuf,
}

//		PublicAssets															
/// Loading configuration for public static assets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
pub struct PublicAssets {
	//		Public properties													
	/// The loading behaviour for local, non-baked-in public static assets. This
	/// allows local assets to be used to complement the baked-in assets.
	#[default(LoadingBehavior::Deny)]
	pub behavior:   LoadingBehavior,
	
	/// The path to the local, non-baked-in public static assets.
	#[default = "static"]
	pub local_path: PathBuf,
}

//		StaticFiles																
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
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


