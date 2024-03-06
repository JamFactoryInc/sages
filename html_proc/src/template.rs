use std::{collections::HashSet, iter};

use proc_macro::TokenStream;

pub struct HtmlTemplate {
    source: String,
}

impl HtmlTemplate {

    pub fn from(source: String) -> HtmlTemplate {
        HtmlTemplate {
            source,
        }
    }

    pub fn as_token_stream(self) -> TokenStream {
        let array_contents = self.source.bytes().map(|b| format!("{b},")).collect::<String>();
        format!("&[{array_contents}]").parse().unwrap()
    }
}