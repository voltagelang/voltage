#[cfg(test)]
mod integration_tests {
    use crate::{Lexer, Parser};
    use voltage_core::Statement;

    #[test]
    fn test_complete_program_parsing() {
        let source = r#"
        fn main() {
            print("Hello, World");
            puts("This is a test");
            let x = 42;
            puts("x is {}", x);
        }
        "#.to_string();
        
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        // Should have tokens
        assert!(!tokens.is_empty());
        
        let mut parser = Parser::new(tokens.to_vec());
        let ast = parser.parse();
        
        // Should have at least one statement (the main function)
        assert!(!ast.is_empty());
        
        // The first statement should be a function
        match &ast[0] {
            Statement::Function(func) => {
                assert_eq!(func.name, "main");
                // Function should have body statements
                assert!(!func.body.is_empty());
            },
            _ => panic!("Expected a function statement"),
        }
    }
    
    #[test]
    fn test_variable_declaration_parsing() {
        let source = r#"let x = 123;"#.to_string();
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().to_vec();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        
        // Should have exactly one statement
        assert_eq!(ast.len(), 1);
    }
    
    #[test]
    fn test_function_call_parsing() {
        let source = r#"puts("test");"#.to_string();
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().to_vec();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        
        // Should have exactly one statement
        assert_eq!(ast.len(), 1);
    }
    
    #[test]
    fn test_format_call_parsing() {
        let source = r#"puts("value is {}", x);"#.to_string();
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().to_vec();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        
        // Should have exactly one statement
        assert_eq!(ast.len(), 1);
    }
}
