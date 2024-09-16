//! Core functionality for the application.



//		Packages

use include_dir::Dir;
use std::sync::Arc;
use tera::Tera;



//		Functions

//		setup_tera																
/// Sets up the Tera template engine.
/// 
/// This function reads all the `.tera.html` files in the specified directory
/// and adds them to the Tera template engine.
/// 
/// # Parameters
/// 
/// * `template_dir` - The directory containing the HTML templates. This is
///                    wrapped inside an [`Arc`] to support reusability across
///                    the application if required.
/// 
#[expect(clippy::expect_used, reason = "Misconfiguration or inability to start, so hard quit")]
pub fn setup_tera(template_dir: &Arc<Dir<'static>>) -> Tera {
	let mut templates = vec![];
	for file in template_dir.find("**/*.tera.html").expect("Failed to read glob pattern") {
		templates.push((
			file.path().file_name().unwrap()
				.to_str().unwrap()
				.strip_suffix(".tera.html").unwrap()
				.to_owned(),
			template_dir.get_file(file.path()).unwrap().contents_utf8().unwrap(),
		));
	}
	let mut tera = Tera::default();
	tera.add_raw_templates(templates).expect("Error parsing templates");
	tera.autoescape_on(vec![".tera.html", ".html"]);
	tera
}


