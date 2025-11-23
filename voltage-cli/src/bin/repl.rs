use std::io::{self, Write};
use voltage_parser::{Lexer, Parser};
use voltage_jit::JitCompiler;

pub struct Repl {
    jit: JitCompiler,
}

impl Repl {
    pub fn new() -> Self {
        let mut jit = JitCompiler::new();
        
        // Declare built-in functions
        if let Err(e) = jit.declare_builtins() {
            eprintln!("Error declaring built-ins: {}", e);
        }
        
        Self { jit }
    }
    
    pub fn run(&mut self) {
        println!("Welcome to the Voltage REPL!");
        println!("Enter Voltage code (type 'exit' to quit)");
        
        loop {
            print!("> ");
            io::stdout().flush().unwrap(); // Make sure the prompt is displayed
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            
            let input = input.trim();
            
            if input == "exit" || input == "quit" {
                println!("Goodbye!");
                break;
            }
            
            if input.is_empty() {
                continue;
            }
            
            // For now, just echo back the input with basic processing
            // In the future, we'd want to parse and execute the code
            match self.process_input(input.to_string()) {
                Ok(result) => {
                    if !result.is_empty() {
                        println!("{}", result);
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }
    
    fn process_input(&mut self, input: String) -> Result<String, String> {
        // Check if this is a function definition or an expression
        let source = if input.trim_end().ends_with(';') {
            // If it ends with semicolon, it's an expression statement
            format!("fn temp() {{ {} }}", input)
        } else {
            // For now, treat everything as needing to be in a function
            format!("fn temp() {{ {}; }}", input)
        };
        
        // Tokenize
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().to_vec();
        
        // Parse
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        
        // Find the function and compile it
        for stmt in ast {
            if let voltage_core::Statement::Function(func) = stmt {
                if func.name == "temp" {
                    // Try to compile the temporary function
                    self.jit.compile_function(&func)?;
                    return Ok("Compiled successfully".to_string());
                }
            }
        }
        
        Ok("Processed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let repl = Repl::new();
        assert!(true); // Basic test that REPL can be created
    }
}