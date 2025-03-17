use oxc_formatter::{JsFormatOptions, format_source};
use oxc_span::SourceType;

fn main() {
    let source_text = "let a, b='Hey';const c =   [2,3,4]   ; call()";
    let source_type = SourceType::mjs();

    let options = JsFormatOptions::default();
    println!("🛠️ Format with options:");
    println!("{options:?}");

    println!("👀 Original code:");
    println!("{source_text}");

    match format_source(source_text, source_type, options) {
        Ok(formatted) => {
            println!("✨ Formatted code:");
            println!("{}", formatted);
        }
        Err(err) => {
            eprintln!("💥 Failed to format:");
            eprintln!("{err}");
        }
    };
}
