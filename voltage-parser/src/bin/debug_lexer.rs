fn main() {
    let source = r#"fn main() { }"#.to_string();
    let lexer = voltage_parser::Lexer::new(source);
    let tokens = lexer.tokenize();
    
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
}