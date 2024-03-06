use std::collections::HashSet;

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
    let mut state = HtmlParseState::Default;
    let mut string = String::with_capacity(contents.len() * 2);
    
    HtmlTemplate::from(contents)
}

fn handle_default(state: &mut ParseState) {

}