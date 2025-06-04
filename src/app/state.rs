//! Application state functionality.



//		Packages																										

use core::net::{IpAddr, SocketAddr};

#[cfg(feature = "tera")]
use super::{
	config::HtmlTemplates,
	errors::AppError,
};
#[cfg(feature = "tera")]
use ::{
	core::future::Future,
	tera::{Context, Tera},
};



//		Traits																											

//§		StateProvider															
/// A trait for providing the application state for general functionality.
pub trait StateProvider: Send + Sync + 'static {
	//		address																
	/// Gets the actual address the server is running on.
	fn address(&self) -> Option<SocketAddr>;
	
	//		html_templates_config												
	/// Gets the configuration for the HTML templates.
	#[cfg(feature = "tera")]
	fn html_templates_config(&self) -> &HtmlTemplates;
	
	//		host																
	/// Gets the configured host, in the form of an IP address.
	fn host(&self) -> IpAddr;
	
	//		port																
	/// Gets the configured port.
	fn port(&self) -> u16;
	
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
	#[cfg(feature = "tera")]
	fn render<T: AsRef<str> + Send>(
		&self,
		template: T,
		context:  &Context,
	) -> impl Future<Output = Result<String, AppError>> + Send;
	
	//		set_address															
	/// Sets the actual address the server is running on.
	/// 
	/// # Parameters
	/// 
	/// * `address` - The actual address the server is running on.
	/// 
	fn set_address(&self, address: Option<SocketAddr>);
	
	//		tera																
	/// Gets the application's Tera instance.
	#[cfg(feature = "tera")]
	fn tera(&self) -> &Tera;
	
	//		title																
	/// Gets the application title.
	fn title(&self) -> &String;
}


