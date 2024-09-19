//! Application state functionality.



//		Packages

use tera::{Context, Error as TemplateError};



//		Traits

//§		StateProvider															
/// A trait for providing the application state for general functionality.
pub trait StateProvider: Send + Sync + 'static {
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


