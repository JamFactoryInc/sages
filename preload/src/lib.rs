use std::{collections::HashSet, fs::File, io::Read, path::Path};

use proc_macro::TokenStream;

mod template;
mod processor;

#[proc_macro]
pub fn preload(stream: TokenStream) -> TokenStream {
    // stream should just be a file location relative to /page
    let stream_string = stream.to_string();

    let is_valid_file_location = stream_string.starts_with("\"")
            && stream_string.ends_with("\"")
            && stream_string.len() >= 3;
    let file_location =  if is_valid_file_location {
        &stream_string[1..stream_string.len() - 1]
    } else {
        let err_msg = format!("Invalid input {stream_string}. Argument must be a string literal");
        return err(&err_msg)
    };

    embed(file_location)
}

fn err(msg: &str) -> TokenStream {
    format!("compile_error!(\"{}\")", msg.replace("\\", "\\\\").replace("\"", "\\\"")).parse().unwrap()
}

fn embed(location: &str) -> TokenStream {
    let absolute_location = format!("{}/../page/{location}", env!("CARGO_MANIFEST_DIR"));
    let path = Path::new(&absolute_location);
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return err(&absolute_location)
    };
    
    let mut file_contents = String::new();
    if let Err(e) = file.read_to_string(&mut file_contents) {
        return err(&e.to_string());
    }
    
    let mut embed_path = HashSet::with_capacity(1);
    embed_path.insert(location.to_string());
    let template = processor::process(path.extension(), file_contents, embed_path);

    template.as_token_stream()
}