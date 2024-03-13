use std::{collections::HashSet, io::Write};

use libflate::gzip;

use minify_html::Cfg;

use crate::template::HtmlTemplate;


enum HtmlParseState {
    Default,
    InHtmlOpenTag,
    InHtmlOpenTagIdent,
    HtmlOpenTagExpect
}

struct ParseState {
    
}

pub fn parse_html(contents: String, embed_path: HashSet<String>) -> HtmlTemplate {
    let minified = minify_html::minify(contents.as_bytes(), &Cfg::spec_compliant());
    HtmlTemplate::from(String::from_utf8(minified).expect(""))
    let encoded = Vec::with_capacity(contents.as_bytes().len());
    let mut encoder = gzip::Encoder::new(encoded).unwrap();
    encoder.write_all(contents.as_bytes()).unwrap();
    let result = encoder.finish();
    let encoded = result.into_result().unwrap();
    HtmlTemplate::from(encoded)
}

fn handle_default(state: &mut ParseState) {

}