use std::collections::HashSet;

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
}

fn handle_default(state: &mut ParseState) {

}