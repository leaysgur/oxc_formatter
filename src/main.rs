use oxc_formatter::format_source;
use oxc_span::SourceType;

fn main() {
    let source_text = "let a, b=1;const c =   [2,3,4]";
    let source_type = SourceType::mjs();
    println!("👀 Original code:");
    println!("{source_text}");

    let result = format_source(source_text, source_type);
    assert!(result.is_ok());

    println!("✨ Formatted code:");
    println!("{}", result.unwrap());
}
