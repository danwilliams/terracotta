//! Utility functions and types for the application.



//		Packages

#[cfg(feature = "tera")]
use super::{
	config::LoadingBehavior,
	errors::AppError,
	state::StateProvider,
};
#[cfg(feature = "tera")]
use ::{
	std::fs,
	tera::Context,
};



//		Functions

//		render																	
/// Renders a template.
/// 
/// Renders a template with the given context and returns the result.
/// 
/// If the application has been configured to allow template overrides, the
/// local filesystem will be searched, and any matching templates found will be
/// used in preference to the baked-in ones.
/// 
/// # Parameters
/// 
/// * `state`    - The application state.
/// * `template` - The name of the template to render.
/// * `context`  - The context to render the template with.
/// 
/// # Errors
/// 
/// If the template cannot be loaded or rendered, an error will be returned.
/// 
#[cfg(feature = "tera")]
pub fn render<SP>(
	state:    &SP,
	template: &str,
	context:  &Context,
) -> Result<String, AppError>
where
	SP: StateProvider,
{
	let local_template = state.html_templates_config().local_path.join(format!("{template}.tera.html"));
	let local_layout   = state.html_templates_config().local_path.join("layout.tera.html");
	let mut tera       = state.tera().clone();
	if state.html_templates_config().behavior == LoadingBehavior::Override {
		if local_layout.exists() {
			tera.add_raw_template(
				"layout",
				&fs::read_to_string(&local_layout)
					.map_err(|err| AppError::CouldNotLoadTemplate(local_layout.clone(), err))?
			).map_err(|err| AppError::CouldNotAddTemplate(local_layout, err))?;
		};
		if local_template.exists() {
			tera.add_raw_template(
				template,
				&fs::read_to_string(&local_template)
					.map_err(|err| AppError::CouldNotLoadTemplate(local_template.clone(), err))?
			).map_err(|err| AppError::CouldNotAddTemplate(local_template, err))?;
		};
	};
	Ok(tera.render(template, context)?)
}


