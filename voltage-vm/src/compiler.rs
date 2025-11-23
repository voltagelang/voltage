use crate::vm::{Bytecode, RuntimeValue};
use voltage_core::{Statement, Expression, Literal, BinaryOp, Function};

pub struct BytecodeCompiler {
    bytecode: Vec<Bytecode>,
    constants: Vec<RuntimeValue>,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
        }
    }

    fn add_constant(&mut self, value: RuntimeValue) -> usize {
        // For now, just add every constant - we can optimize later
        let index = self.constants.len();
        self.constants.push(value);
        index
    }

    pub fn compile_function(&mut self, func: &Function) -> Result<(Vec<Bytecode>, Vec<RuntimeValue>), String> {
        // For now, let's compile a simple example
        // In a real implementation, we'd walk the AST
        
        for stmt in &func.body {
            self.compile_statement(stmt)?;
        }
        
        // Add return null if needed
        let const_index = self.add_constant(RuntimeValue::Integer(0));
        self.bytecode.push(Bytecode::LoadConst(const_index));
        self.bytecode.push(Bytecode::Return);
        
        Ok((self.bytecode.clone(), self.constants.clone()))
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                // Pop the result since expressions as statements don't return anything
                self.bytecode.push(Bytecode::Pop);
            }
            Statement::VariableDeclaration { name, value, explicit_type: _ } => {
                // Compile the value
                self.compile_expression(value)?;
                // For now, we'll just pop it since we don't have proper local variable handling
                self.bytecode.push(Bytecode::Pop);
                println!("Compiling variable declaration: {}", name);
            }
            Statement::Block(statements) => {
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
            }
            Statement::Function(_) => {
                return Err("Nested functions not implemented yet".to_string());
            }
            Statement::If { condition, then_branch, elif_branches, else_branch } => {
                // For now, let's just compile all branches without actual control flow
                // In a real implementation, we'd use conditional jumps
                self.compile_expression(condition)?;
                for stmt in then_branch {
                    self.compile_statement(stmt)?;
                }
                for (elif_condition, elif_body) in elif_branches {
                    self.compile_expression(elif_condition)?;
                    for stmt in elif_body {
                        self.compile_statement(stmt)?;
                    }
                }
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        self.compile_statement(stmt)?;
                    }
                }
            }
            Statement::While { condition, body } => {
                // For now, just compile without actual looping
                // In a real implementation, we'd use jumps for the loop
                self.compile_expression(condition)?;
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
            }
            Statement::For { iterable, body, .. } => {
                // For now, just compile without actual iteration
                // In a real implementation, we'd handle iteration properly
                self.compile_expression(iterable)?;
                for stmt in body {
                    self.compile_statement(stmt)?;
                }
            }
            Statement::Break | Statement::Continue => {
                // These would be handled in a proper loop context
                // For now, just compile as no-op
            }
            Statement::UnsafeBlock(statements) => {
                // For now, just compile the contents of the unsafe block
                for stmt in statements {
                    self.compile_statement(stmt)?;
                }
            }
            Statement::Import(module_name) => {
                // In a full implementation, this would load the specified module
                // For now, just compile to a no-op
                // Add the imported module to the module registry
                println!("Importing module: {}", module_name);
            }
            Statement::ImportAs(module_name, alias) => {
                // Import module with an alias
                // In a full implementation, register the module with the given alias
                println!("Importing module: {} as {}", module_name, alias);
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Literal(literal) => {
                let value = self.literal_to_runtime_value(literal)?;
                let index = self.add_constant(value);
                self.bytecode.push(Bytecode::LoadConst(index));
            }
            Expression::VariableDeclaration { .. } => {
                return Err("VariableDeclaration expression not expected in this context".to_string());
            }
            Expression::Variable(name) => {
                // For now, assume it's a global variable
                self.bytecode.push(Bytecode::LoadGlobal(name.clone()));
            }
            Expression::Binary { left, operator, right } => {
                // Compile left operand
                self.compile_expression(left)?;
                // Compile right operand
                self.compile_expression(right)?;
                // Apply operator
                match operator {
                    BinaryOp::Add => self.bytecode.push(Bytecode::Add),
                    BinaryOp::Subtract => self.bytecode.push(Bytecode::Sub),
                    BinaryOp::Multiply => self.bytecode.push(Bytecode::Mul),
                    BinaryOp::Divide => self.bytecode.push(Bytecode::Div),
                    BinaryOp::Modulo => self.bytecode.push(Bytecode::Mod), // Add Modulo operation
                    BinaryOp::Equal => self.bytecode.push(Bytecode::Eq),
                    BinaryOp::NotEqual => self.bytecode.push(Bytecode::Ne),
                    BinaryOp::Less => self.bytecode.push(Bytecode::Lt),
                    BinaryOp::LessEqual => self.bytecode.push(Bytecode::Le),
                    BinaryOp::Greater => self.bytecode.push(Bytecode::Gt),
                    BinaryOp::GreaterEqual => self.bytecode.push(Bytecode::Ge),
                }
            }
            Expression::Call { name, arguments } => {
                // Compile arguments (push them on stack)
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                
                // Handle built-in functions specially
                match name.as_str() {
                    "print" => {
                        if arguments.len() == 1 {
                            self.bytecode.push(Bytecode::CallBuiltin(1));
                        } else {
                            return Err("print function expects 1 argument".to_string());
                        }
                    }
                    "puts" => {
                        if arguments.len() == 1 {
                            self.bytecode.push(Bytecode::CallBuiltin(0));
                        } else {
                            return Err("puts function expects 1 argument".to_string());
                        }
                    }
                    _ => {
                        // For user-defined functions, push the function name and call
                        // In a more complete implementation, we'd have function lookup
                        let func_name_const = self.add_constant(RuntimeValue::String(name.clone()));
                        self.bytecode.push(Bytecode::LoadConst(func_name_const));
                        self.bytecode.push(Bytecode::Call(arguments.len()));
                    }
                }
            }
            Expression::FormatCall { name, format_string, arguments } => {
                // For formatted calls, we need to compile all arguments
                // For now, simplify by just taking the first argument (which should be the format string)
                // and push all argument values for potential formatting
                println!("Format call: {} with format string: {} and {} args", name, format_string, arguments.len());
                
                // Compile format string as constant
                let fmt_value = RuntimeValue::String(format_string.clone());
                let fmt_index = self.add_constant(fmt_value);
                
                // Compile all arguments
                for arg in arguments {
                    self.compile_expression(arg)?;
                }
                
                // For now, compile as a regular builtin call since we don't have full formatting
                // In a complete implementation, we'd handle format string substitution
                match name.as_str() {
                    "puts" => {
                        self.bytecode.push(Bytecode::CallBuiltin(0)); // puts builtin
                    }
                    "print" => {
                        self.bytecode.push(Bytecode::CallBuiltin(1)); // print builtin
                    }
                    _ => {
                        return Err(format!("Unknown function: {}", name));
                    }
                }
            }
            Expression::ArrayLiteral(elements) => {
                // For now, compile array literals by pushing each element
                // In a real implementation, we'd create an actual array object
                for element in elements {
                    self.compile_expression(element)?;
                }
                // For now, just push a placeholder indicating an array was created
                let array_placeholder = RuntimeValue::String(format!("array_{}", elements.len()));
                let index = self.add_constant(array_placeholder);
                self.bytecode.push(Bytecode::LoadConst(index));
            },
            Expression::ArrayAccess { array, index } => {
                // For now, just compile the array and index expressions
                // In a real implementation, we'd access the array element
                self.compile_expression(array)?;
                self.compile_expression(index)?;
                // For now, push a placeholder value
                let const_idx = self.add_constant(RuntimeValue::Integer(0));
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
            Expression::ArrayAssignment { array, index, value } => {
                // For now, compile all components
                // In a real implementation, we'd assign to the array element
                self.compile_expression(array)?;
                self.compile_expression(index)?;
                self.compile_expression(value)?;
                // Pop everything since assignment doesn't return a value
                self.bytecode.push(Bytecode::Pop);
                self.bytecode.push(Bytecode::Pop);
                self.bytecode.push(Bytecode::Pop);
                // Push null for no return value
                let const_idx = self.add_constant(RuntimeValue::Null);
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
            Expression::StructDefinition { .. } => {
                // Struct definitions are compile-time constructs, no runtime code needed
                let const_idx = self.add_constant(RuntimeValue::Null);
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
            Expression::StructInitialization { fields, .. } => {
                // For now, compile field values
                // In a real implementation, we'd create a struct instance
                for (_, value) in fields {
                    self.compile_expression(value)?;
                }
                // Push a placeholder struct value
                let struct_value = RuntimeValue::String("struct_instance".to_string());
                let const_idx = self.add_constant(struct_value);
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
            Expression::StructFieldAccess { object, field } => {
                // Compile the object
                self.compile_expression(object)?;
                // For now, push a placeholder
                // In reality, we'd access the field from the object
                let field_name = RuntimeValue::String(field.clone());
                let const_idx = self.add_constant(field_name);
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
            Expression::StructFieldAssignment { object, field, value } => {
                // Compile the object and value
                self.compile_expression(object)?;
                self.compile_expression(value)?;
                // In reality, we'd assign the value to the field
                // For now, pop the values and return null
                self.bytecode.push(Bytecode::Pop); // pop value
                self.bytecode.push(Bytecode::Pop); // pop object
                let field_name = RuntimeValue::String(field.clone());
                let const_idx = self.add_constant(field_name);
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
            Expression::EnumVariantCreation { variant_name, values, .. } => {
                // Compile the enum variant creation
                // For now, just push values
                for value in values {
                    self.compile_expression(value)?;
                }
                // Push a placeholder enum value
                let enum_value = RuntimeValue::String(format!("enum_{}", variant_name));
                let const_idx = self.add_constant(enum_value);
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
            Expression::EnumMatch { .. } => {
                // For now, just return null
                // In a real implementation, we'd handle pattern matching
                let const_idx = self.add_constant(RuntimeValue::Null);
                self.bytecode.push(Bytecode::LoadConst(const_idx));
            },
        }
        Ok(())
    }

    fn literal_to_runtime_value(&self, literal: &Literal) -> Result<RuntimeValue, String> {
        match literal {
            Literal::Integer(n) => Ok(RuntimeValue::Integer(*n)),
            Literal::Float(f) => Ok(RuntimeValue::Float(*f)),
            Literal::String(s) => Ok(RuntimeValue::String(s.clone())),
            Literal::Boolean(b) => Ok(RuntimeValue::Boolean(*b)),
        }
    }
}