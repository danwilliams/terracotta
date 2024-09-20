//! Application state functionality.



//		Packages

use core::net::{IpAddr, SocketAddr};
use tera::{Context, Error as TemplateError};



//		Traits

//§		StateProvider															
/// A trait for providing the application state for general functionality.
pub trait StateProvider: Send + Sync + 'static {
	//		address																
	/// Gets the actual address the server is running on.
	fn address(&self) -> Option<SocketAddr>;
	
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
	fn render<T: AsRef<str>>(&self, template: T, context: &Context) -> Result<String, TemplateError>;
	
	//		set_address															
	/// Sets the actual address the server is running on.
	/// 
	/// # Parameters
	/// 
	/// * `address` - The actual address the server is running on.
	/// 
	fn set_address(&self, address: Option<SocketAddr>);
	
	//		title																
	/// Gets the application title.
	fn title(&self) -> &String;
}


