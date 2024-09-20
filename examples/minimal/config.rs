//! Configuration for the application.



//		Packages

use core::net::IpAddr;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;



//		Structs

//		Config																	
/// The main configuration options for the application.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, SmartDefault)]
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
}


