use crate::lexer::{Token, Lexer};
use voltage_core::{Expression, Literal, BinaryOp, Statement, Function};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        
        statements
    }
    
    fn declaration(&mut self) -> Option<Statement> {
        if self.is_at_end() || self.check(&Token::RightBrace) {
            return None;
        }
        
        if self.match_token(&Token::Fn) {
            return Some(self.function_declaration());
        }
        
        if self.match_token(&Token::Let) {
            return Some(self.var_declaration());
        }
        
        // Check if it's the end of the block before attempting to parse a statement
        if self.is_at_end() || self.check(&Token::RightBrace) {
            return None;
        }
        
        self.statement()
    }
    
    fn statement(&mut self) -> Option<Statement> {
        if self.check(&Token::RightBrace) || self.is_at_end() {
            return None;
        }
        
        if self.match_token(&Token::If) {
            return Some(self.if_statement());
        }
        
        if self.match_token(&Token::While) {
            return Some(self.while_statement());
        }
        
        if self.match_token(&Token::For) {
            return Some(self.for_statement());
        }
        
        if self.match_token(&Token::Break) {
            self.consume(&Token::Semi).expect("Expected ';'");
            return Some(Statement::Break);
        }
        
        if self.match_token(&Token::Continue) {
            self.consume(&Token::Semi).expect("Expected ';'");
            return Some(Statement::Continue);
        }
        
        if self.match_token(&Token::Unsafe) {
            self.consume(&Token::LeftBrace).expect("Expected '{' after unsafe block");
            let body = self.parse_block_contents();
            return Some(Statement::UnsafeBlock(body));
        }
        
        if self.match_token(&Token::Import) {
            let module_name = self.consume_identifier().expect("Expected module name after import");
            
            // Check if there's an 'as' alias
            if self.match_token(&Token::As) {
                let alias = self.consume_identifier().expect("Expected alias name after 'as'");
                return Some(Statement::ImportAs(module_name, alias));
            } else {
                return Some(Statement::Import(module_name));
            }
        }
        
        // Also add a call to handle the in token in for loops if not already handled
        // We already handle 'in' in the for_statement method
        
        if self.match_token(&Token::LeftBrace) {
            return Some(self.block().unwrap());
        }

        // Check again after handling block
        if self.check(&Token::RightBrace) || self.is_at_end() {
            return None;
        }

        // Parse expression statement
        let expr = self.expression();
        self.consume(&Token::Semi).expect("Expected ';'");
        Some(Statement::Expression(expr))
    }
    
    fn function_declaration(&mut self) -> Statement {
        let name = self.consume_identifier().expect("Expected function name");
        
        self.consume(&Token::LeftParen).expect("Expected '(' after function name");
        
        let mut parameters = Vec::new();  // This should be a Vec<(String, Type)> to match Function definition
        if !self.check(&Token::RightParen) {
            loop {
                let param_name = self.consume_identifier().expect("Expected parameter name");
                
                // Check if there's a type annotation for this parameter
                let param_type = if self.check(&Token::Colon) {
                    self.consume(&Token::Colon).expect("Expected ':' for parameter type");
                    if let Ok(parsed_type) = self.parse_type() {
                        parsed_type
                    } else {
                        voltage_core::Type::Unknown  // Default to Unknown if parsing fails
                    }
                } else {
                    voltage_core::Type::Unknown  // Will be inferred
                };
                
                parameters.push((param_name, param_type));
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
                
                if self.check(&Token::RightParen) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightParen).expect("Expected ')'");
        
        // Check if there's a return type annotation using '->'
        let return_type = if self.check(&Token::Arrow) {
            self.consume(&Token::Arrow).expect("Expected '->' for return type");
            if let Ok(parsed_type) = self.parse_type() {
                parsed_type
            } else {
                voltage_core::Type::Void  // Default to Void if parsing fails
            }
        } else {
            voltage_core::Type::Void  // Default to void
        };
        
        // At this point, the next token should be the opening brace of the function body
        self.consume(&Token::LeftBrace).expect("Expected '{' for function body");
        
        let body = self.parse_block_contents();
        
        Statement::Function(Function {
            name,
            parameters,
            return_type,
            body,
        })
    }
    
    fn var_declaration(&mut self) -> Statement {
        let name = self.consume_identifier().expect("Expected variable name");
        
        // Check if there's a type annotation
        let explicit_type = if self.check(&Token::Colon) {
            self.consume(&Token::Colon).expect("Expected ':' after variable name");
            if let Ok(parsed_type) = self.parse_type() {
                Some(parsed_type)
            } else {
                // If parsing the type failed, try to continue by skipping ahead
                // For now, we'll just return None but in a real implementation we'd handle this better
                None
            }
        } else {
            None
        };
        
        self.consume(&Token::Equals).expect("Expected '=' after variable name");
        
        let value = self.expression();
        
        self.consume(&Token::Semi).expect("Expected ';' after variable declaration");
        
        Statement::VariableDeclaration {
            name,
            value,
            explicit_type,
        }
    }
    
    fn parse_type(&mut self) -> Result<voltage_core::Type, String> {
        // For now, we'll support basic types
        // Later we might support function types, generics, etc.
        match &self.tokens[self.current] {
            Token::Identifier(type_name) => {
                self.current += 1; // consume the identifier
                
                match type_name.as_str() {
                    "i32" | "int" => Ok(voltage_core::Type::Integer),
                    "f64" | "float" => Ok(voltage_core::Type::Float),
                    "bool" | "boolean" => Ok(voltage_core::Type::Boolean),
                    "str" | "string" => Ok(voltage_core::Type::String),
                    "void" => Ok(voltage_core::Type::Void),
                    _ => Err(format!("Unknown type: {}", type_name)),
                }
            },
            _ => Err("Expected type identifier".to_string()),
        }
    }
    

    
    fn parse_block_contents(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                // No statement could be parsed, likely reached end of block
                break;
            }
        }
        
        self.consume(&Token::RightBrace).expect("Expected '}'");
        
        statements
    }
    
    // Keep the block function for inline blocks like { stmt; }
    fn block(&mut self) -> Option<Statement> {
        // This function is meant to handle inline blocks that start with '{'
        // First consume the left brace
        self.consume(&Token::LeftBrace).unwrap(); // This is called from statement which already matched LeftBrace
        
        let statements = self.parse_block_contents();
        Some(Statement::Block(statements))
    }
    
    fn expression(&mut self) -> Expression {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Expression {
        let expr = self.equality();
        
        if self.match_token(&Token::Equals) {
            // For now, we'll just return the expression as-is since we don't handle assignments yet
            // This is just a placeholder to avoid compilation errors
        }
        
        expr
    }
    
    fn equality(&mut self) -> Expression {
        let mut expr = self.comparison();
        
        while self.match_token(&Token::Equal) || 
              self.match_token(&Token::NotEqual) {
            let operator = match self.previous_token() {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => panic!("Unexpected operator"),
            };
            
            let right = self.comparison();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn comparison(&mut self) -> Expression {
        let mut expr = self.term();
        
        while self.match_token(&Token::Less) || 
              self.match_token(&Token::LessEqual) || 
              self.match_token(&Token::Greater) || 
              self.match_token(&Token::GreaterEqual) {
            let operator = match self.previous_token() {
                Token::Less => BinaryOp::Less,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::Greater => BinaryOp::Greater,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => panic!("Unexpected operator"),
            };
            
            let right = self.term();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn term(&mut self) -> Expression {
        let mut expr = self.factor();
        
        while self.match_token(&Token::Plus) || 
              self.match_token(&Token::Minus) {
            let operator = match self.previous_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => panic!("Unexpected operator"),
            };
            
            let right = self.factor();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn factor(&mut self) -> Expression {
        let mut expr = self.unary();
        
        while self.match_token(&Token::Star) || 
              self.match_token(&Token::Slash) ||
              self.match_token(&Token::Percent) {
            let operator = match self.previous_token() {
                Token::Star => BinaryOp::Multiply,
                Token::Slash => BinaryOp::Divide,
                Token::Percent => BinaryOp::Modulo,
                _ => panic!("Unexpected operator"),
            };
            
            let right = self.unary();
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        expr
    }
    
    fn unary(&mut self) -> Expression {
        self.call()
    }
    
    fn call(&mut self) -> Expression {
        let mut expr = self.primary();
        
        loop {
            if self.match_token(&Token::LeftParen) {
                expr = self.finish_call(expr);
            } else if self.match_token(&Token::LeftBracket) {
                // Handle array access: array[index]
                let index = self.expression();
                self.consume(&Token::RightBracket).expect("Expected ']'");
                expr = Expression::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&Token::Dot) {
                // Handle field access: obj.field or method calls
                if let Token::Identifier(field_name) = self.tokens[self.current].clone() {
                    self.current += 1;
                    
                    // Check if this is a method call (with parentheses)
                    if self.match_token(&Token::LeftParen) {
                        // This would be a method call - for now just handle as field access
                        // In a future implementation, this could be extended to method calls
                    }
                    
                    expr = Expression::StructFieldAccess {
                        object: Box::new(expr),
                        field: field_name,
                    };
                } else {
                    panic!("Expected field name after '.'");
                }
            } else {
                break;
            }
        }
        
        expr
    }
    
    fn finish_call(&mut self, callee: Expression) -> Expression {
        let mut arguments = Vec::new();
        
        if !self.check(&Token::RightParen) {
            loop {
                arguments.push(self.expression());
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightParen).expect("Expected ')'");
        
        if let Expression::Variable(name) = callee {
            // Check if this is a format string call (print/puts with {} formatting)
            if (name == "puts" || name == "print") && !arguments.is_empty() {
                // For format string calls like puts("the value of x is {}", x)
                // We need to check if the first argument contains {}
                if let Expression::Literal(Literal::String(ref format_str)) = &arguments[0] {
                    if format_str.contains("{}") {
                        return Expression::FormatCall {
                            name,
                            format_string: format_str.clone(),
                            arguments: arguments[1..].to_vec(),
                        };
                    }
                }
            }
            
            Expression::Call {
                name,
                arguments,
            }
        } else {
            panic!("Only variable names can be called (got {:?})", callee);
        }
    }
    
    fn primary(&mut self) -> Expression {
        if self.current >= self.tokens.len() {
            panic!("Unexpected end of input");
        }

        // Handle array literals: [expr, expr, ...]
        if self.match_token(&Token::LeftBracket) {
            let mut elements = Vec::new();
            
            if !self.check(&Token::RightBracket) {
                loop {
                    elements.push(self.expression());
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                    
                    if self.check(&Token::RightBracket) {
                        break;
                    }
                }
            }
            
            self.consume(&Token::RightBracket).expect("Expected ']'");
            return Expression::ArrayLiteral(elements);
        }
        
        // Handle grouped expressions: (expr)
        if self.match_token(&Token::LeftParen) {
            let expr = self.expression();
            self.consume(&Token::RightParen).expect("Expected ')'");
            return expr;
        }
        
        // Handle literals and identifiers
        if let Token::Number(n) = &self.tokens[self.current] {
            self.current += 1;
            return Expression::Literal(Literal::Integer(*n));
        }
        
        if let Token::String(s) = &self.tokens[self.current] {
            self.current += 1;
            return Expression::Literal(Literal::String(s.clone()));
        }
        
        // Handle boolean literals
        if let Token::Identifier(name) = &self.tokens[self.current] {
            if name == "true" {
                self.current += 1;
                return Expression::Literal(Literal::Boolean(true));
            } else if name == "false" {
                self.current += 1;
                return Expression::Literal(Literal::Boolean(false));
            } else {
                // Regular identifier - check for special syntax before consuming it
                let identifier_name = name.clone();
                
                // Check if this is a struct initialization: Name { field: value }
                // We look ahead to see if next token is LeftBrace
                if self.current + 1 < self.tokens.len() && self.tokens[self.current + 1] == Token::LeftBrace {
                    self.current += 1;  // Consume the identifier
                    return self.struct_initialization(identifier_name);
                }
                
                // Check if this is an enum variant: EnumName::Variant(...) or EnumName::Variant
                // We look ahead to see if next token is DoubleColon
                if self.current + 1 < self.tokens.len() && self.tokens[self.current + 1] == Token::DoubleColon {
                    self.current += 1;  // Consume the identifier
                    return self.enum_variant_creation(identifier_name);
                }
                
                // Regular variable usage
                self.current += 1;
                return Expression::Variable(identifier_name);
            }
        }
        
        // If we reach here, we didn't match any known expression form
        panic!("Expected expression, got {:?}", self.tokens[self.current])
    }
    
    fn struct_initialization(&mut self, struct_name: String) -> Expression {
        // Expect opening brace
        self.consume(&Token::LeftBrace).expect("Expected '{' for struct initialization");
        
        let mut fields = Vec::new();
        
        if !self.check(&Token::RightBrace) {
            loop {
                let field_name = self.consume_identifier().expect("Expected field name in struct initialization");
                self.consume(&Token::Colon).expect("Expected ':' in struct initialization");
                let field_value = self.expression();
                
                fields.push((field_name, field_value));
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
                
                if self.check(&Token::RightBrace) {
                    break;
                }
            }
        }
        
        self.consume(&Token::RightBrace).expect("Expected '}' after struct initialization");
        
        Expression::StructInitialization {
            name: struct_name,
            fields,
        }
    }
    
    fn enum_variant_creation(&mut self, enum_name: String) -> Expression {
        // Parse EnumName::Variant format
        self.consume(&Token::DoubleColon).expect("Expected '::' for enum variant");
        let variant_name = self.consume_identifier().expect("Expected variant name");
        
        // If followed by parentheses, it has values; otherwise it's a unit variant
        let values = if self.match_token(&Token::LeftParen) {
            let mut args = Vec::new();
            
            if !self.check(&Token::RightParen) {
                loop {
                    args.push(self.expression());
                    
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                    
                    if self.check(&Token::RightParen) {
                        break;
                    }
                }
            }
            
            self.consume(&Token::RightParen).expect("Expected ')'");
            args
        } else {
            Vec::new()  // Unit variant with no values
        };
        
        Expression::EnumVariantCreation {
            enum_name,
            variant_name,
            values,
        }
    }
    
    fn consume(&mut self, token: &Token) -> Result<(), String> {
        if self.check(token) {
            self.current += 1;
            Ok(())
        } else {
            let current_token_str = if self.is_at_end() {
                "EOF".to_string()
            } else {
                format!("{:?}", self.current_token())
            };
            Err(format!("Expected {:?}, got {}", token, current_token_str))
        }
    }
    
    fn consume_identifier(&mut self) -> Result<String, String> {
        if self.is_at_end() {
            return Err("Expected identifier, got EOF".to_string());
        }
        
        if let Token::Identifier(name) = &self.tokens[self.current] {
            self.current += 1;
            Ok(name.clone())
        } else {
            Err(format!("Expected identifier, got {:?}", self.current_token()))
        }
    }
    
    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.current += 1;
            true
        } else {
            false
        }
    }
    
    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            matches!(&self.tokens[self.current], t if std::mem::discriminant(t) == std::mem::discriminant(token))
        }
    }
    
    fn advance_if_identifier(&mut self) -> Option<String> {
        if let Token::Identifier(name) = &self.tokens[self.current] {
            self.current += 1;
            Some(name.clone())
        } else {
            None
        }
    }
    
    fn previous_token(&self) -> &Token {
        if self.current == 0 {
            panic!("No previous token available");
        }
        &self.tokens[self.current - 1]
    }
    
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
    
    fn if_statement(&mut self) -> Statement {
        // Parse the condition
        let condition = self.expression();
        
        // Expect the opening brace for the then branch
        self.consume(&Token::LeftBrace).expect("Expected '{' after if condition");
        let then_branch = self.parse_block_contents();
        
        // Check for elif branches
        let mut elif_branches = Vec::new();
        while self.match_token(&Token::Elif) {
            let elif_condition = self.expression();
            self.consume(&Token::LeftBrace).expect("Expected '{' after elif condition");
            let elif_body = self.parse_block_contents();
            elif_branches.push((elif_condition, elif_body));
        }
        
        // Check for else branch
        let else_branch = if self.match_token(&Token::Else) {
            self.consume(&Token::LeftBrace).expect("Expected '{' after else");
            Some(self.parse_block_contents())
        } else {
            None
        };
        
        Statement::If {
            condition,
            then_branch,
            elif_branches,
            else_branch,
        }
    }
    
    fn while_statement(&mut self) -> Statement {
        let condition = self.expression();
        self.consume(&Token::LeftBrace).expect("Expected '{' after while condition");
        let body = self.parse_block_contents();
        
        Statement::While {
            condition,
            body,
        }
    }
    
    fn for_statement(&mut self) -> Statement {
        let variable = self.consume_identifier().expect("Expected variable name in for loop");
        
        // Expect 'in' token
        self.consume(&Token::In).expect("Expected 'in' in for loop");
        
        let iterable = self.expression();
        self.consume(&Token::LeftBrace).expect("Expected '{' after for loop");
        let body = self.parse_block_contents();
        
        Statement::For {
            variable,
            iterable,
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = r#"fn main() { }"#.to_string();
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize().to_vec();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        
        assert!(!ast.is_empty());
        // Additional assertions can be added here
    }
}