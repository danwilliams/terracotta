//! Application state functionality.



//		Packages

use crate::config::Config;
use core::net::{IpAddr, SocketAddr};
use include_dir::include_dir;
use parking_lot::RwLock;
use std::sync::Arc;
use tera::{Context, Tera};
use terracotta::app::{
	config::HtmlTemplates,
	errors::AppError,
	init::setup_tera,
	state::StateProvider as AppStateProvider,
	utility::render,
};



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
	pub address: RwLock<Option<SocketAddr>>,
	
	/// The application configuration.
	pub config:  Config,
	
	/// The Tera template engine.
	pub tera:    Tera,
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
	
	//		html_templates_config												
	fn html_templates_config(&self) -> &HtmlTemplates {
		&self.config.html
	}
	
	//		port																
	fn port(&self) -> u16 {
		self.config.port
	}
	
	//		render																
	async fn render<T: AsRef<str> + Send>(&self, template: T, context: &Context) -> Result<String, AppError> {
		render(self, template.as_ref(), context).await
	}
	
	//		set_address															
	fn set_address(&self, address: Option<SocketAddr>) {
		*self.address.write() = address;
	}
	
	//		tera																
	fn tera(&self) -> &Tera {
		&self.tera
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
			address: RwLock::new(None),
			config:  Config::default(),
			tera:    setup_tera(&Arc::new(include_dir!("examples/resources/html")))
				.expect("Error loading templates")
			,
		}
	}
}


