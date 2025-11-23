#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Integer,
    Float,
    String,
    Boolean,
    Void,
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    Array(Box<Type>, usize),
    DynamicArray(Box<Type>),
    Slice(Box<Type>),
    Pointer(Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Struct(String, Vec<(String, Type)>),
    Enum(String, Vec<(String, Option<Vec<Type>>)>),
    Generic(String),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TypedExpression {
    pub expression: Expression,
    pub type_info: Type,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    VariableDeclaration {
        name: String,
        value: Box<Expression>,
        explicit_type: Option<Type>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Call {
        name: String,
        arguments: Vec<Expression>,
    },
    FormatCall {
        name: String,
        format_string: String,
        arguments: Vec<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    ArrayAssignment {
        array: Box<Expression>,
        index: Box<Expression>,
        value: Box<Expression>,
    },
    StructDefinition {
        name: String,
        fields: Vec<(String, Type)>,
    },
    StructInitialization {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    StructFieldAccess {
        object: Box<Expression>,
        field: String,
    },
    StructFieldAssignment {
        object: Box<Expression>,
        field: String,
        value: Box<Expression>,
    },
    EnumVariantCreation {
        enum_name: String,
        variant_name: String,
        values: Vec<Expression>,
    },
    EnumMatch {
        expression: Box<Expression>,
        arms: Vec<(EnumPattern, Expression)>,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub struct TypedFunction {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<TypedStatement>,
}

#[derive(Debug, Clone)]
pub enum TypedStatement {
    Expression(TypedExpression),
    VariableDeclaration {
        name: String,
        value: TypedExpression,
        declared_type: Type,
    },
    Block(Vec<TypedStatement>),
    Function(TypedFunction),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    VariableDeclaration {
        name: String,
        value: Expression,
        explicit_type: Option<Type>,
    },
    Block(Vec<Statement>),
    Function(Function),
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        elif_branches: Vec<(Expression, Vec<Statement>)>,
        else_branch: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Break,
    Continue,
    UnsafeBlock(Vec<Statement>),
    Import(String),
    ImportAs(String, String),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum EnumPattern {
    Variant(String, Option<Vec<String>>),
    Wildcard,
    Literal(Literal),
}