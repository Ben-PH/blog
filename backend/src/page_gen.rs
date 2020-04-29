
lazy_static::lazy_static! {
    pub (crate) static ref TEMPLATES: tera::Tera = {
        match tera::Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*.html")) {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        }
    };
}
