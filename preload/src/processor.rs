use std::{collections::HashSet, ffi::OsStr, io::Write};

use libflate::gzip;

use minify_html::Cfg;

use crate::template::HtmlTemplate;

pub fn process(file_extension: Option<&OsStr>, contents: String, embed_path: HashSet<String>) -> HtmlTemplate {
    let minified = match file_extension.and_then(OsStr::to_str) {
        // js is minified by webpack
        Some("css" | "html") => {
            let mut minify_config = Cfg::spec_compliant();
            minify_config.minify_css = true;
            minify_config.minify_js = true;
            minify_html::minify(contents.as_bytes(), &minify_config)
        },
        // don't try to minify file types we don't recognise
        _ => Vec::from_iter(contents.bytes())
    };

    let mut encoder = gzip::Encoder::new(Vec::with_capacity(contents.as_bytes().len())).unwrap();
    encoder.write_all(minified.as_slice()).unwrap();
    let result = encoder.finish();
    let encoded = result.into_result().unwrap();
    HtmlTemplate::from(encoded)
}