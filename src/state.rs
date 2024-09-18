//! Application state functionality.



//		Packages

use crate::{
	assets::{
		config::AssetsConfig,
		state::StateProvider as AssetsStateProvider,
	},
	auth::state::StateProvider as AuthStateProvider,
	config::Config,
	stats::{
		config::StatsConfig,
		state::{AppStateStats, StateProvider as StatsStateProvider},
	},
};
use include_dir::Dir;
use std::{
	collections::HashMap,
	sync::Arc,
};
use tera::{Context, Error as TemplateError, Tera};
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
	pub stats:       RwLock<AppStateStats>,
	
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
	//		assets_config														
	fn assets_config(&self) -> &AssetsConfig {
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
	//		stats_config														
	fn stats_config(&self) -> &StatsConfig {
		&self.config.stats
	}
	
	//		stats_state															
	fn stats_state(&self) -> &RwLock<AppStateStats> {
		&self.stats
	}
}



//		Traits

//§		AppStateProvider														
/// A trait for providing the application state for general functionality.
pub trait AppStateProvider: Send + Sync + 'static {
	//		render																
	/// Renders a template.
	/// 
	/// # Parameters
	/// 
	/// * `template` - The template to render.
	/// * `context`  - The context to render the template with.
	/// 
	/// # Errors
	/// 
	/// If the template cannot be rendered, an error is returned.
	/// 
	fn render<T: AsRef<str>>(&self, template: T, context: &Context) -> Result<String, TemplateError>;
	
	//		title																
	/// Gets the application title.
	fn title(&self) -> &String;
}


