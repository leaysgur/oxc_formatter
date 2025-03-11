use oxc_formatter::format_source;
use oxc_span::SourceType;

fn main() {
    let source_text = "let a, b=1;const c =   [2,3,4]   ; call()";
    let source_type = SourceType::mjs();
    println!("👀 Original code:");
    println!("{source_text}");

    match format_source(source_text, source_type) {
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
