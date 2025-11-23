use clap::Parser as ClapParser;
use voltage_core::*;
use voltage_parser::{Parser, Lexer};
use voltage_jit::JitCompiler;
use voltage_vm::{VirtualMachine, BytecodeCompiler};
use std::fs;

mod repl;

#[derive(ClapParser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to compile and run
    #[arg(value_name = "FILE")]
    input: Option<String>,
    
    /// Run in REPL mode
    #[arg(long)]
    repl: bool,
}

fn main() {
    let cli = Cli::parse();
    
    if cli.repl {
        // Run REPL mode
        let mut repl_instance = repl::Repl::new();
        repl_instance.run();
        return;
    }
    
    match &cli.input {
        Some(file) => {
            if file.ends_with(".v") {
                run_voltage_file(file);
            } else {
                println!("Compiling file: {}", file);
                
                // Read the source code from the file
                let source = fs::read_to_string(file)
                    .expect("Should have been able to read the file");
                    
                // Tokenize the source
                let lexer = Lexer::new(source);
                let tokens = lexer.tokenize().to_vec();
                println!("Tokens: {:?}", tokens);
                
                // Parse the tokens into AST
                let mut parser = Parser::new(tokens);
                let ast = parser.parse();
                println!("Parsed {} statements", ast.len());
                
                // Set up the JIT compiler
                let mut jit = JitCompiler::new();
                
                // Declare built-in functions
                if let Err(e) = jit.declare_builtins() {
                    eprintln!("Error declaring built-ins: {}", e);
                    return;
                }
                
                // Compile each top-level function in the AST
                for stmt in ast {
                    match stmt {
                        Statement::Function(func) => {
                            println!("Compiling function: {}", func.name);
                            if let Err(e) = jit.compile_function(&func) {
                                eprintln!("Error compiling function '{}': {}", func.name, e);
                            }
                        }
                        _ => {
                            println!("Skipping non-function statement");
                        }
                    }
                }
                
                println!("Compilation completed successfully!");
            }
        }
        None => {
            println!("Voltage programming language");
            println!("Usage: voltage [OPTIONS] [FILE]");
            println!("  voltage file.v         Compile and run a .v file with Voltage Engine");
            println!("  voltage file.vx        Compile with legacy JIT (for comparison)");
            println!("  voltage --repl         Run in REPL mode");
            
            // Example of the syntax
            println!("\nExample syntax:");
            println!("fn main() {{");
            println!("   print(\"this print function doesnt include a newline automatically at the end\");");
            println!("   puts(\"this puts function includes a newline at the end\");");
            println!("   let x = 1;");
            println!("   puts(\"the value of x is {{}}\", x);");
            println!("}}");
        }
    }
}

fn run_voltage_file(file: &str) {
    println!("Running Voltage file: {}", file);
    
    // Read the source code from the file
    let source = fs::read_to_string(file)
        .expect("Should have been able to read the file");
        
    // Tokenize the source
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    
    // Parse the tokens into AST
    let mut parser = Parser::new(tokens.to_vec());
    let ast = parser.parse();
    
    // Find and execute the main function
    for stmt in ast {
        if let Statement::Function(func) = stmt {
            if func.name == "main" {
                // Compile to bytecode using the Voltage Engine
                let mut compiler = BytecodeCompiler::new();
                match compiler.compile_function(&func) {
                    Ok((bytecode, constants)) => {
                        println!("Successfully compiled to bytecode!");
                        println!("Bytecode length: {}", bytecode.len());
                        println!("Constants count: {}", constants.len());
                        
                        // Create and run the VM
                        let mut vm = VirtualMachine::new();
                        vm.load_bytecode(bytecode, constants);
                        
                        match vm.run() {
                            Ok(result) => println!("Program completed with result: {:?}", result),
                            Err(e) => eprintln!("Runtime error: {}", e),
                        }
                        return; // Successfully executed
                    }
                    Err(e) => {
                        eprintln!("Compilation error: {}", e);
                        return;
                    }
                }
            }
        }
    }
    
    eprintln!("No main function found in {}", file);
}