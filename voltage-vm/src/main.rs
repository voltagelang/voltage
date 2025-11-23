use voltage_vm::{VirtualMachine, BytecodeCompiler};
use voltage_parser::{Lexer, Parser};
use voltage_core::Function;

fn main() {
    // Example Voltage code
    let source = r#"fn main() { let x = 42; puts("Hello from Voltage VM!"); }"#;

    // Parse the source code
    let lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens.to_vec());
    let ast = parser.parse();
    
    // Find the main function
    for stmt in ast {
        if let voltage_core::Statement::Function(func) = stmt {
            if func.name == "main" {
                // Compile to bytecode
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
                            Ok(result) => println!("Program result: {:?}", result),
                            Err(e) => eprintln!("Runtime error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Compilation error: {}", e),
                }
                break;
            }
        }
    }
}