use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use cranelift_jit::{JITBuilder, JITModule};
use voltage_core::{Expression, Statement, Function, Literal};

pub struct JitCompiler {
    builder_context: FunctionBuilderContext,
    module: JITModule,
}

impl JitCompiler {
    pub fn new() -> Self {
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).expect("Failed to create JITBuilder");
        let module = JITModule::new(builder);
        
        Self {
            builder_context: FunctionBuilderContext::new(),
            module,
        }
    }
    
    pub fn compile_function(&mut self, func: &Function) -> Result<(), String> {
        // Create a signature for the function
        let mut sig = self.module.make_signature();
        
        // For now, assuming all functions return i32
        sig.returns.push(AbiParam::new(types::I32));
        
        // Create the function
        let func_id = self.module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;
            
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        
        // Process the function body to extract information we need for compilation
        let has_builtin_calls = self.function_has_builtin_calls(func);
        
        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_context);
            
            // Create the entry block
            let block = builder.create_block();
            builder.append_block_params_for_function_params(block);
            builder.switch_to_block(block);
            builder.seal_block(block);
            
            // If the function contains builtin calls, we'll need more complex logic
            if has_builtin_calls {
                // For now, just return 0
                let return_val = builder.ins().iconst(types::I32, 0);
                builder.ins().return_(&[return_val]);
            } else {
                // For functions without builtin calls, return 0
                let return_val = builder.ins().iconst(types::I32, 0);
                builder.ins().return_(&[return_val]);
            }
        }
        
        // Compile the function
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| e.to_string())?;
            
        let _ = self.module.finalize_definitions();
        
        Ok(())
    }
    
    // Helper to check if a function contains built-in calls
    fn function_has_builtin_calls(&self, func: &Function) -> bool {
        for stmt in &func.body {
            if self.statement_has_builtin_call(stmt) {
                return true;
            }
        }
        false
    }
    
    fn statement_has_builtin_call(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::Expression(expr) => self.expression_has_builtin_call(expr),
            Statement::Block(statements) => {
                for stmt in statements {
                    if self.statement_has_builtin_call(stmt) {
                        return true;
                    }
                }
                false
            },
            Statement::VariableDeclaration { value, .. } => self.expression_has_builtin_call(value),
            Statement::Function(nested_func) => self.function_has_builtin_calls(nested_func),
            Statement::If { condition, then_branch, elif_branches, else_branch } => {
                // Check condition for builtin calls
                if self.expression_has_builtin_call(condition) {
                    return true;
                }
                // Check then branch
                for stmt in then_branch {
                    if self.statement_has_builtin_call(stmt) {
                        return true;
                    }
                }
                // Check elif branches
                for (cond, body) in elif_branches {
                    if self.expression_has_builtin_call(cond) {
                        return true;
                    }
                    for stmt in body {
                        if self.statement_has_builtin_call(stmt) {
                            return true;
                        }
                    }
                }
                // Check else branch
                if let Some(branches) = else_branch {
                    for stmt in branches {
                        if self.statement_has_builtin_call(stmt) {
                            return true;
                        }
                    }
                }
                false
            },
            Statement::While { condition, body } => {
                if self.expression_has_builtin_call(condition) {
                    return true;
                }
                for stmt in body {
                    if self.statement_has_builtin_call(stmt) {
                        return true;
                    }
                }
                false
            },
            Statement::For { iterable, body, .. } => {
                if self.expression_has_builtin_call(iterable) {
                    return true;
                }
                for stmt in body {
                    if self.statement_has_builtin_call(stmt) {
                        return true;
                    }
                }
                false
            },
            Statement::Break | Statement::Continue => false,
            Statement::UnsafeBlock(statements) => {
                for stmt in statements {
                    if self.statement_has_builtin_call(stmt) {
                        return true;
                    }
                }
                false
            },
            Statement::Import(_) | Statement::ImportAs(_, _) => {
                // Import statements themselves don't have builtin calls,
                // but the imported modules might use them
                false
            },
        }
    }
    
    fn expression_has_builtin_call(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Call { name, .. } => name == "print" || name == "puts",
            Expression::FormatCall { name, .. } => name == "print" || name == "puts", 
            Expression::Binary { left, right, .. } => {
                self.expression_has_builtin_call(left) || self.expression_has_builtin_call(right)
            },
            _ => false,
        }
    }
    
    // Method to declare external functions like print and puts
    pub fn declare_builtins(&mut self) -> Result<(), String> {
        // Declare print function (for print without newline)
        {
            let mut sig = self.module.make_signature();
            // For now, just take a pointer to string
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I32));
            
            self.module
                .declare_function("print", Linkage::Import, &sig)
                .map_err(|e| e.to_string())?;
        }
        
        // Declare puts function (for puts with newline)
        {
            let mut sig = self.module.make_signature();
            // For now, just take a pointer to string
            sig.params.push(AbiParam::new(types::I64));
            sig.returns.push(AbiParam::new(types::I32));
            
            self.module
                .declare_function("puts", Linkage::Import, &sig)
                .map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }
    
    // Enhanced compile_function to handle variable declarations
    pub fn compile_function_advanced(&mut self, func: &Function) -> Result<(), String> {
        // Create a signature for the function
        let mut sig = self.module.make_signature();
        
        // For now, assuming all functions return i32
        sig.returns.push(AbiParam::new(types::I32));
        
        // Create the function
        let func_id = self.module
            .declare_function(&func.name, Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;
            
        let mut ctx = self.module.make_context();
        ctx.func.signature = sig;
        
        {
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut self.builder_context);
            
            // Create the entry block
            let block = builder.create_block();
            builder.append_block_params_for_function_params(block);
            builder.switch_to_block(block);
            builder.seal_block(block);
            
            // Process the function body (we'll need to implement this properly)
            // For now, just return 0
            let return_val = builder.ins().iconst(types::I32, 0);
            builder.ins().return_(&[return_val]);
        }
        
        // Compile the function
        self.module
            .define_function(func_id, &mut ctx)
            .map_err(|e| e.to_string())?;
            
        let _ = self.module.finalize_definitions();
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_compiler_creation() {
        let _compiler = JitCompiler::new();
        // Basic test to ensure JIT compiler can be created
        assert!(true); // This will always pass, but ensures no panic on creation
    }
    
    #[test]
    fn test_builtin_declaration() {
        let mut compiler = JitCompiler::new();
        assert!(compiler.declare_builtins().is_ok());
    }
}