use std::collections::HashMap;
use voltage_core::Literal;

#[derive(Debug, Clone)]
pub enum Bytecode {
    // Constants and variables
    LoadConst(usize),           // Load constant from constant pool
    StoreLocal(usize),          // Store to local variable
    LoadLocal(usize),           // Load from local variable
    StoreGlobal(String),        // Store to global variable
    LoadGlobal(String),         // Load from global variable

    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,  // Modulo operation

    // Comparison operations
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    // Control flow
    Jump(usize),                // Unconditional jump
    JumpIfFalse(usize),         // Jump if top of stack is false
    JumpIfTrue(usize),          // Jump if top of stack is true
    Call(usize),                // Call function (arg = num args)
    CallBuiltin(usize),         // Call builtin function (arg = builtin id)
    Return,                     // Return from function

    // Built-in functions
    Print,
    Puts,

    // Stack operations
    Pop,
    Dup,
}

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Function { name: String, ip: usize, num_params: usize }, // Function with bytecode position
    Null,
}

// Implement PartialEq manually to handle floats properly
impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a == b,
            (RuntimeValue::Float(a), RuntimeValue::Float(b)) => (a - b).abs() < f64::EPSILON,
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
            (RuntimeValue::Boolean(a), RuntimeValue::Boolean(b)) => a == b,
            (RuntimeValue::Function { name: a, .. }, RuntimeValue::Function { name: b, .. }) => a == b,
            (RuntimeValue::Null, RuntimeValue::Null) => true,
            _ => false,
        }
    }
}

// We'll avoid using RuntimeValue as a HashMap key for floats by using indices instead
// The compiler module will handle constant deduplication differently

pub struct VirtualMachine {
    bytecode: Vec<Bytecode>,
    constants: Vec<RuntimeValue>,
    stack: Vec<RuntimeValue>,
    globals: HashMap<String, RuntimeValue>,
    ip: usize,  // Instruction pointer
    // For now, function locations will be stored in constants or we'll implement function mapping
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
            stack: Vec::new(),
            globals: HashMap::new(),
            ip: 0,
        }
    }

    pub fn load_bytecode(&mut self, bytecode: Vec<Bytecode>, constants: Vec<RuntimeValue>) {
        self.bytecode = bytecode;
        self.constants = constants;
        self.ip = 0;
    }

    pub fn run(&mut self) -> Result<RuntimeValue, String> {
        loop {
            if self.ip >= self.bytecode.len() {
                break;
            }

            let instruction = self.bytecode[self.ip].clone();
            self.ip += 1;

            match instruction {
                Bytecode::LoadConst(index) => {
                    let value = self.constants[index].clone();
                    self.stack.push(value);
                }
                Bytecode::Add => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                            self.stack.push(RuntimeValue::Integer(a + b));
                        }
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                            self.stack.push(RuntimeValue::Float(a + b));
                        }
                        _ => return Err("Type error: Cannot add non-numeric values".to_string()),
                    }
                }
                Bytecode::Sub => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                            self.stack.push(RuntimeValue::Integer(a - b));
                        }
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                            self.stack.push(RuntimeValue::Float(a - b));
                        }
                        _ => return Err("Type error: Cannot subtract non-numeric values".to_string()),
                    }
                }
                Bytecode::Mul => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                            self.stack.push(RuntimeValue::Integer(a * b));
                        }
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                            self.stack.push(RuntimeValue::Float(a * b));
                        }
                        _ => return Err("Type error: Cannot multiply non-numeric values".to_string()),
                    }
                }
                Bytecode::Div => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                            if b == 0 {
                                return Err("Division by zero".to_string());
                            }
                            self.stack.push(RuntimeValue::Integer(a / b));
                        }
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                            if b == 0.0 {
                                return Err("Division by zero".to_string());
                            }
                            self.stack.push(RuntimeValue::Float(a / b));
                        }
                        _ => return Err("Type error: Cannot divide non-numeric values".to_string()),
                    }
                }
                Bytecode::Mod => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => {
                            if b == 0 {
                                return Err("Modulo by zero".to_string());
                            }
                            self.stack.push(RuntimeValue::Integer(a % b));
                        }
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => {
                            if b == 0.0 {
                                return Err("Modulo by zero".to_string());
                            }
                            self.stack.push(RuntimeValue::Float(a % b));
                        }
                        _ => return Err("Type error: Cannot perform modulo on non-numeric values".to_string()),
                    }
                }
                Bytecode::Eq => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    let result = left == right;
                    self.stack.push(RuntimeValue::Boolean(result));
                }
                Bytecode::Ne => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    let result = left != right;
                    self.stack.push(RuntimeValue::Boolean(result));
                }
                Bytecode::Lt => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    let result = match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a < b,
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a < b,
                        _ => return Err("Type error: Cannot compare non-numeric values".to_string()),
                    };
                    self.stack.push(RuntimeValue::Boolean(result));
                }
                Bytecode::Gt => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    let result = match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a > b,
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a > b,
                        _ => return Err("Type error: Cannot compare non-numeric values".to_string()),
                    };
                    self.stack.push(RuntimeValue::Boolean(result));
                }
                Bytecode::Le => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    let result = match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a <= b,
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a <= b,
                        _ => return Err("Type error: Cannot compare non-numeric values".to_string()),
                    };
                    self.stack.push(RuntimeValue::Boolean(result));
                }
                Bytecode::Ge => {
                    let right = self.pop_value()?;
                    let left = self.pop_value()?;
                    let result = match (left, right) {
                        (RuntimeValue::Integer(a), RuntimeValue::Integer(b)) => a >= b,
                        (RuntimeValue::Float(a), RuntimeValue::Float(b)) => a >= b,
                        _ => return Err("Type error: Cannot compare non-numeric values".to_string()),
                    };
                    self.stack.push(RuntimeValue::Boolean(result));
                }
                Bytecode::Print => {
                    let value = self.pop_value()?;
                    print!("{}", self.value_to_string(&value));
                }
                Bytecode::Puts => {
                    let value = self.pop_value()?;
                    println!("{}", self.value_to_string(&value));
                }
                Bytecode::StoreLocal(index) => {
                    let value = self.pop_value()?;
                    // For now, we'll just store in a temporary place
                    // In a full implementation, we'd have proper local variable storage
                    println!("Storing local var {}: {:?}", index, value);
                }
                Bytecode::LoadLocal(index) => {
                    // For now, just push a dummy value
                    // In a full implementation, we'd load from proper local storage
                    println!("Loading local var {}", index);
                }
                Bytecode::Call(num_args) => {
                    // For now, we'll handle user function calls by name
                    // Pop the function name (for direct calls)
                    let func_name_value = self.pop_value()?;
                    
                    if let RuntimeValue::String(func_name) = func_name_value {
                        match func_name.as_str() {
                            "puts" => {
                                if num_args == 1 {
                                    let arg = self.pop_value()?;
                                    println!("{}", self.value_to_string(&arg));
                                    self.stack.push(RuntimeValue::Null);
                                } else {
                                    return Err("puts expects 1 argument".to_string());
                                }
                            }
                            "print" => {
                                if num_args == 1 {
                                    let arg = self.pop_value()?;
                                    print!("{}", self.value_to_string(&arg));
                                    self.stack.push(RuntimeValue::Null);
                                } else {
                                    return Err("print expects 1 argument".to_string());
                                }
                            }
                            _ => {
                                return Err(format!("Unknown function: {}", func_name));
                            }
                        }
                    } else {
                        return Err("Function call expects function name as string".to_string());
                    }
                }
                Bytecode::CallBuiltin(builtin_id) => {
                    // Handle builtin functions by ID
                    match builtin_id {
                        0 => { // puts
                            let arg = self.pop_value()?;
                            println!("{}", self.value_to_string(&arg));
                            self.stack.push(RuntimeValue::Null);
                        }
                        1 => { // print
                            let arg = self.pop_value()?;
                            print!("{}", self.value_to_string(&arg));
                            self.stack.push(RuntimeValue::Null);
                        }
                        _ => return Err(format!("Unknown builtin function ID: {}", builtin_id)),
                    }
                }
                Bytecode::Return => {
                    // For now, just pop the return value and continue
                    // In a real implementation with call frames, this would restore the previous frame
                    let _ = self.pop_value().unwrap_or(RuntimeValue::Null); // Pop return value if any
                    // For now, we'll continue execution, but in a real implementation
                    // this would return to the caller. We'll just continue until end.
                }
                Bytecode::Pop => {
                    self.stack.pop();
                }
                Bytecode::LoadGlobal(name) => {
                    // Try to get from globals, or return an error
                    match self.globals.get(&name) {
                        Some(value) => {
                            self.stack.push(value.clone());
                        }
                        None => {
                            // If not found in globals, it might be undefined
                            // For now return a default value
                            self.stack.push(RuntimeValue::Integer(0));
                        }
                    }
                }
                Bytecode::StoreGlobal(name) => {
                    let value = self.pop_value()?;
                    self.globals.insert(name, value);
                }
                // More instructions will be added later...
                _ => {
                    return Err(format!("Unsupported instruction: {:?}", instruction));
                }
            }
        }

        // Return the top of the stack or null if empty
        Ok(self.stack.pop().unwrap_or(RuntimeValue::Null))
    }

    fn pop_value(&mut self) -> Result<RuntimeValue, String> {
        self.stack.pop().ok_or_else(|| "Stack underflow".to_string())
    }

    fn value_to_string(&self, value: &RuntimeValue) -> String {
        match value {
            RuntimeValue::Integer(i) => i.to_string(),
            RuntimeValue::Float(f) => f.to_string(),
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Boolean(b) => b.to_string(),
            RuntimeValue::Function { name, .. } => format!("<function {}>", name),
            RuntimeValue::Null => "null".to_string(),
        }
    }
}