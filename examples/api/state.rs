//! Application state functionality.



//		Packages

use crate::config::Config;
use core::net::{IpAddr, SocketAddr};
use include_dir::include_dir;
use parking_lot::RwLock;
use std::sync::Arc;
use tera::{Context, Error as TemplateError, Tera};
use terracotta::{
	app::{
		init::setup_tera,
		state::StateProvider as AppStateProvider,
	},
	stats::{
		config::Config as StatsConfig,
		state::{State as StatsState, StateProvider as StatsStateProvider},
	},
};
use tokio::sync::RwLock as AsyncRwLock;



//		Structs

//		AppState																
/// The application state.
/// 
/// This is used to store global state information that is shared between
/// requests.
/// 
#[derive(Debug)]
pub struct AppState {
	//		Public properties													
	/// The address the server is running on.
	pub address:     RwLock<Option<SocketAddr>>,
	
	/// The application configuration.
	pub config:      Config,
	
	/// The application statistics.
	pub stats:       AsyncRwLock<StatsState>,
	
	/// The Tera template engine.
	pub template:    Tera,
}

//󰭅		AppState																
impl AppState {
	//		new																	
	/// Creates a new application state.
	/// 
	/// # Parameters
	/// 
	/// * `config` - The application configuration.
	/// 
	/// # Returns
	/// 
	/// The new application state.
	/// 
	pub fn new(config: Config) -> Self {
		Self {
			config,
			..Default::default()
		}
	}
}

//󰭅		AppStateProvider														
impl AppStateProvider for AppState {
	//		address																
	fn address(&self) -> Option<SocketAddr> {
		*self.address.read()
	}
	
	//		host																
	fn host(&self) -> IpAddr {
		self.config.host
	}
	
	//		port																
	fn port(&self) -> u16 {
		self.config.port
	}
	
	//		render																
	fn render<T: AsRef<str>>(&self, template: T, context: &Context) -> Result<String, TemplateError> {
		self.template.render(template.as_ref(), context)
	}
	
	//		set_address															
	fn set_address(&self, address: Option<SocketAddr>) {
		*self.address.write() = address;
	}
	
	//		title																
	fn title(&self) -> &String {
		&self.config.title
	}
}

//󰭅		Default																	
impl Default for AppState {
	//		default																
	fn default() -> Self {
		Self {
			address:  RwLock::new(None),
			config:   Config::default(),
			stats:    AsyncRwLock::new(StatsState::default()),
			template: setup_tera(&Arc::new(include_dir!("html"))).expect("Error loading templates"),
		}
	}
}

//󰭅		StatsStateProvider														
impl StatsStateProvider for AppState {
	//		config																
	fn config(&self) -> &StatsConfig {
		&self.config.stats
	}
	
	//		state																
	fn state(&self) -> &AsyncRwLock<StatsState> {
		&self.stats
	}
}


