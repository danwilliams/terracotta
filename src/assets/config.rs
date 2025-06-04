#![allow(clippy::exhaustive_structs, reason = "Configuration structs")]

//! Configuration for the assets-serving functionality.



//		Packages																										

use crate::app::config::LoadingBehavior;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::path::PathBuf;



//		Structs																											

//		Config																	
/// The configuration options for gathering and processing statistics.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
pub struct Config {
	//		Public properties													
	/// Loading configuration for protected static assets.
	#[serde(rename = "protected")]
	pub protected_assets: ProtectedAssets,
	
	/// Loading configuration for public static assets.
	#[serde(rename = "public")]
	pub public_assets:    PublicAssets,
	
	/// The configuration options for serving static files.
	pub static_files:     StaticFiles,
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


