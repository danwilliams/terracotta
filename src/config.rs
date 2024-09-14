//! Configuration for the application.



//		Packages

use crate::{
	assets::config::AssetsConfig,
	stats::config::StatsConfig,
};
use core::net::IpAddr;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::collections::HashMap;



//		Structs

//		Config																	
/// The main configuration options for the application.
#[derive(Deserialize, Serialize, SmartDefault)]
pub struct Config {
	//		Public properties													
	/// The host to listen on.
	#[default(IpAddr::from([127, 0, 0, 1]))]
	pub host:   IpAddr,
	
	/// The port to listen on.
	#[default = 8000]
	pub port:   u16,
	
	/// The directory to store log files in.
	#[default = "log"]
	pub logdir: String,
	
	/// The title of the application.
	#[default = "Terracotta"]
	pub title:  String,
	
	/// The configuration options for serving static assets.
	pub assets: AssetsConfig,
	
	/// The configuration options for gathering and processing statistics.
	pub stats:  StatsConfig,
	
	/// A list of users and their passwords.
	#[default(HashMap::new())]
	pub users:  HashMap<String, String>,
}


