use logos::Logos;

#[derive(Logos, Clone, Debug, PartialEq)]
pub enum Token {
    #[token("fn")]
    Fn,
    
    #[token("let")]
    Let,
    
    #[token("if")]
    If,
    
    #[token("else")]
    Else,
    
    #[token("elif")]
    Elif,
    
    #[token("for")]
    For,
    
    #[token("while")]
    While,
    
    #[token("break")]
    Break,
    
    #[token("continue")]
    Continue,
    
    #[token("in")]
    In,
    
    #[token("unsafe")]
    Unsafe,
    
    #[token("import")]
    Import,
    
    #[token("as")]
    As,
    
    #[token("=")]
    Equals,
    
    #[token(":")]
    Colon,
    
    #[token("(")]
    LeftParen,
    
    #[token(")")]
    RightParen,
    
    #[token("{")]
    LeftBrace,
    
    #[token("}")]
    RightBrace,
    
    #[token(";")]
    Semi,
    
    #[token(",")]
    Comma,
    
    #[token("->")]
    Arrow,
    
    #[token(">")]
    Greater,
    
    #[token(">=")]
    GreaterEqual,
    
    #[token("<")]
    Less,
    
    #[token("<=")]
    LessEqual,
    
    #[token("==")]
    Equal,
    
    #[token("!=")]
    NotEqual,
    
    #[token("+")]
    Plus,
    
    #[token("-")]
    Minus,
    
    #[token("*")]
    Star,
    
    #[token("/")]
    Slash,
    
    #[token("%")]
    Percent,
    
    #[token("[")]
    LeftBracket,
    
    #[token("]")]
    RightBracket,
    
    #[token(".")]
    Dot,
    
    #[token("::")]
    DoubleColon,
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    
    #[regex(r"[0-9]+", |lex| lex.slice().parse().unwrap_or(0))]
    Number(i64),
    
    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().to_string())]
    String(String),
    
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
}

#[derive(Debug)]
pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let mut lexer = Token::lexer(&source);
        let mut tokens = Vec::new();
        
        while let Some(token_result) = lexer.next() {
            match token_result {
                Ok(token) => tokens.push(token),
                Err(_) => {
                    // Log the error for debugging but continue processing
                    eprintln!("Lexer error: Could not tokenize a portion of the source");
                    // Skip the problematic part and continue
                    continue;
                }
            }
        }
        
        Self { source, tokens }
    }
    
    pub fn tokenize(&self) -> &[Token] {
        &self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let source = r#"fn main() { let x = 1; }"#.to_string();
        let lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Identifier("main".to_string()));
        assert_eq!(tokens[2], Token::LeftParen);
        assert_eq!(tokens[3], Token::RightParen);
        assert_eq!(tokens[4], Token::LeftBrace);
        assert_eq!(tokens[5], Token::Let);
        assert_eq!(tokens[6], Token::Identifier("x".to_string()));
        assert_eq!(tokens[7], Token::Equals);
        assert_eq!(tokens[8], Token::Number(1));
        assert_eq!(tokens[9], Token::Semi);
        assert_eq!(tokens[10], Token::RightBrace);
    }
}