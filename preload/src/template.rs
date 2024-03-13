use proc_macro::TokenStream;

pub struct HtmlTemplate {
    source: Vec<u8>,
}

impl HtmlTemplate {

    pub fn from(source: Vec<u8>) -> HtmlTemplate {
        HtmlTemplate {
            source,
        }
    }

    pub fn as_token_stream(self) -> TokenStream {
        let array_contents = self.source.iter()
            .map(|b| format!("{b},"))
            .collect::<String>();
        format!("&[{array_contents}]").parse().unwrap()
    }
}