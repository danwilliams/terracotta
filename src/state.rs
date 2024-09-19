//! Application state functionality.



//		Packages

use crate::config::Config;
use include_dir::Dir;
use std::{
	collections::HashMap,
	sync::Arc,
};
use tera::{Context, Error as TemplateError, Tera};
use terracotta::{
	app::state::StateProvider as AppStateProvider,
	assets::{
		config::Config as AssetsConfig,
		state::StateProvider as AssetsStateProvider,
	},
	auth::state::StateProvider as AuthStateProvider,
	stats::{
		config::Config as StatsConfig,
		state::{State as StatsState, StateProvider as StatsStateProvider},
	},
};
use tokio::sync::RwLock;



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
	/// The directory containing the static assets.
	pub assets_dir:  Arc<Dir<'static>>,
	
	/// The application configuration.
	pub config:      Config,
	
	/// The directory containing the Markdown content.
	pub content_dir: Arc<Dir<'static>>,
	
	/// The application statistics.
	pub stats:       RwLock<StatsState>,
	
	/// The Tera template engine.
	pub template:    Tera,
}

//󰭅		AppStateProvider														
impl AppStateProvider for AppState {
	//		render																
	fn render<T: AsRef<str>>(&self, template: T, context: &Context) -> Result<String, TemplateError> {
		self.template.render(template.as_ref(), context)
	}
	
	//		title																
	fn title(&self) -> &String {
		&self.config.title
	}
}

//󰭅		AssetsStateProvider														
impl AssetsStateProvider for AppState {
	//		config																
	fn config(&self) -> &AssetsConfig {
		&self.config.assets
	}
	
	//		assets_dir															
	fn assets_dir(&self) -> Arc<Dir<'static>> {
		Arc::clone(&self.assets_dir)
	}
	
	//		content_dir															
	fn content_dir(&self) -> Arc<Dir<'static>> {
		Arc::clone(&self.content_dir)
	}
}

//󰭅		AuthStateProvider														
impl AuthStateProvider for AppState {
	//		users																
	fn users(&self) -> &HashMap<String, String> {
		&self.config.users
	}
}

//󰭅		StatsStateProvider														
impl StatsStateProvider for AppState {
	//		config																
	fn config(&self) -> &StatsConfig {
		&self.config.stats
	}
	
	//		state																
	fn state(&self) -> &RwLock<StatsState> {
		&self.stats
	}
}


