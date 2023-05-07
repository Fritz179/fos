use std::fmt::Write;

#[derive(Debug)]
struct Identifier {
    location: Location,
    identifier: String,
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.identifier)
    }
}

impl std::cmp::PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

#[derive(PartialEq)]
enum Token {
    OpenParen,
    ClosedParen,
    Module,
    Function,
    Parameter,
    Local,
    I32,
    LocalGet,
    I32Add,
    Result,
    Identifier(Identifier),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::OpenParen => f.write_char('('),
            Token::ClosedParen => f.write_char(')'),
            Token::Module => f.write_str("Module"),
            Token::Function => f.write_str("Function"),
            Token::Local => f.write_str("Local"),
            Token::Parameter => f.write_str("Parameter"),
            Token::Result => f.write_str("Result"),
            Token::I32 => f.write_str("i32"),
            Token::LocalGet => f.write_str("local.get"),
            Token::I32Add => f.write_str("i32.add"),
            Token::Identifier(identifier) => f.write_fmt(format_args!("Identifier: {}, at: {}", identifier, identifier.location))
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct Location {
    row: usize,
    colum: usize,
    index: usize
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.row, self.colum))
    }
}

struct Lexer {
    input: Vec<char>,
    location: Location,
    next_tokens: Vec<Token>
}

enum ParseError {
    EndOfFile,
    Unexpected {
        expected: String,
        got: Token
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::EndOfFile => f.write_str("Reached end of file"),
            ParseError::Unexpected { expected, got } => {
                f.write_fmt(format_args!("Expected: '{}' but got: '{}'", expected, got))
            }
        }
    }
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            location: Location { row: 0, colum: 0, index: 0 },
            next_tokens: vec![]
        }
    }

    fn step_location(&mut self, char: char) {
        self.location.index += 1;

        if char == '\n' {
            self.location.colum = 0;
            self.location.row += 1;
        } else {
            self.location.colum += 1;
        }
    }

    fn next_char(&mut self) -> Result<char, ParseError> {
        let char = *self.input.get(self.location.index).ok_or(ParseError::EndOfFile)?;
        
        self.step_location(char);

        Ok(char)
    }

    fn next_char_if(&mut self, predicate: impl FnOnce(char) -> bool) -> Result<Option<char>, ParseError> {
        let char = *self.input.get(self.location.index).ok_or(ParseError::EndOfFile)?;

        if predicate(char) {
            self.step_location(char);

            Ok(Some(char))
        } else {
            Ok(None)
        }
    }

    fn next_token(&mut self) -> Result<Token, ParseError> {
        if !self.next_tokens.is_empty() {
            return Ok(self.next_tokens.pop().unwrap())
        }

        loop {
            let current_location = self.location;
        
            let char = self.next_char()?;
            
            let token = match char {
                '(' => Token::OpenParen,
                ')' => Token::ClosedParen,
                '$' => {
                    let mut token = '$'.to_string();

                    while let Some(next) = self.next_char_if(|c| c.is_alphanumeric())? {
                        token.push(next);
                    }

                    Token::Identifier(Identifier { location: current_location, identifier: token })
                },
                c if c.is_whitespace() => continue,
                c if c.is_alphabetic() => {
                    let mut token = c.to_string();

                    while let Some(next) = self.next_char_if(|c| c.is_alphanumeric() || c == '.')? {
                        token.push(next);
                    }

                    match &token as &str {
                        "module" => Token::Module,
                        "func" => Token::Function,
                        "result" => Token::Result,
                        "param" => Token::Parameter,
                        "local" => Token::Local,
                        "i32" => Token::I32,
                        "local.get" => Token::LocalGet,
                        "i32.add" => Token::I32Add,
                        _ => {
                            println!("Unexpected token: {token}");
                            panic!()
                        }
                    }
                },
                _ => panic!()   
            };

            return Ok(token)
        }
    }

    pub fn return_token(&mut self, token: Token) {
        self.next_tokens.push(token);
    }

    pub fn next_token_if(&mut self, expect: Token) -> Result<bool, ParseError> {
        let token = self.next_token()?;

        if token == expect {
            Ok(true)
        } else {
            self.next_tokens.push(token);
            Ok(false)
        }
    }

    pub fn expect_token(&mut self, expect: Token) -> Result<(), ParseError> {
        let next = self.next_token()?;

        if next == expect {
            Ok(())
        } else {
            Err(ParseError::Unexpected { expected: format!("{expect}").to_string(), got: next })
        }
    }
}

#[derive(Debug)]
enum Node {
    Module(Box<Vec<Node>>),
    Function {
        params: Vec<FunctionDef>,
        locals: Vec<FunctionDef>,
        name: Identifier,
        result: Option<DataType>,
        content: Vec<Instruction>
    },
    Identifier(Identifier)
}

// impl std::fmt::Display for Node {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         f.write_str("Node")
//     }
// }

#[derive(Debug)]
enum DataType {
    i32
}

#[derive(Debug)]
struct FunctionDef {
    identifier: Identifier,
    data_type: DataType,
}

enum FunctionDefs {
    Parameter(FunctionDef),
    Result(DataType),
    Local(FunctionDef),
}

fn parse_data_type(lexer: &mut Lexer) -> Result<DataType, ParseError> {
    let data_type = lexer.next_token()?;
    match &data_type {
        Token::I32 => Ok(DataType::i32),
        _ => Err(ParseError::Unexpected { expected: "datatype".to_string(), got: data_type })
    }
}

fn parse_name(lexer: &mut Lexer) -> Result<Identifier, ParseError> {
    let name = lexer.next_token()?;
    if let Token::Identifier(id) = name {
        Ok(id)
    } else {
        Err(ParseError::Unexpected { expected: "invalid parameter name".to_string(), got: name})
    }
}

fn parse_function_def(lexer: &mut Lexer) -> Result<Option<FunctionDefs>, ParseError> {
    if lexer.next_token_if(Token::OpenParen)? {
        let token = lexer.next_token()?;

        let def = match &token {
            Token::Local => Ok(Some(FunctionDefs::Local(FunctionDef{identifier: parse_name(lexer)?, data_type: parse_data_type(lexer)?}))),
            Token::Result => Ok(Some(FunctionDefs::Result(parse_data_type(lexer)?))),
            Token::Parameter => Ok(Some(FunctionDefs::Parameter(FunctionDef{identifier: parse_name(lexer)?, data_type: parse_data_type(lexer)?}))),
            _ => Err(ParseError::Unexpected { expected: "function parameter".to_string(), got: token })
        };

        lexer.expect_token(Token::ClosedParen)?;

        def
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
enum Instruction {
    LocalGet(Identifier),
    I32Add,
}

fn parse_function_instruction(lexer: &mut Lexer) -> Result<Option<Instruction>, ParseError> {
    let token = lexer.next_token()?;

    let inst = match token {
        Token::LocalGet => Instruction::LocalGet(parse_name(lexer)?),
        Token::I32Add => Instruction::I32Add,
        _ => {
            lexer.return_token(token);
            return Ok(None)
        }
    };

    Ok(Some(inst))
}

fn parse_function(lexer: &mut Lexer) -> Result<Option<Node>, ParseError> {
    if lexer.next_token_if(Token::OpenParen)? {
        if lexer.next_token_if(Token::Function)? {
            match lexer.next_token()? {
                Token::Identifier(name) => {
                    let mut params = vec![];
                    let mut locals = vec![];
                    let mut result = None;

                    while let Some(def) = parse_function_def(lexer)? {
                        match def {
                            FunctionDefs::Local(def) => locals.push(def),
                            FunctionDefs::Parameter(def) => params.push(def),
                            FunctionDefs::Result(def) => {
                                if result.is_some() {
                                    panic!()
                                }

                                result = Some(def)
                            },
                        }
                    };

                    let mut instructions = vec![];

                    loop {
                        let instruction = parse_function_instruction(lexer)?;

                        if let Some(instruction) = instruction  {
                            instructions.push(instruction)
                        } else {
                            break
                        }
                    }

                    lexer.expect_token(Token::ClosedParen)?;


                    Ok(Some(Node::Function { params, locals, name, result, content: instructions }))
                },
                got => Err(ParseError::Unexpected { expected: "function name".to_string(), got })
            }
        } else {
            lexer.return_token(Token::OpenParen);
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn parse(lexer: &mut Lexer) -> Result<Node, ParseError> {
    lexer.expect_token(Token::OpenParen)?;
    lexer.expect_token(Token::Module)?;

    let mut module = Box::new(vec![]);

    loop {
        if lexer.next_token_if(Token::ClosedParen)? {
            break
        }

        let mut node = None;


        if node.is_none() { node = parse_function(lexer)? }


        if let Some(node) = node {
            module.push(node)
        }
        // module.push(node.ok_or(ParseError::Unexpected { expected: "function or stuff".to_string(), got: lexer.next_token()? })?);
    }

    Ok(Node::Module(module))
}

fn main() {
    let input = "(module
      (func $add (param $lhs i32) (param $rhs i32) (result i32)
        local.get $lhs
        local.get $rhs
        i32.add)
    )";

    let mut lexer = Lexer::new(input);

    let tree = parse(&mut lexer);
    
    match tree {
        Ok(tree) => println!("{:#?}", tree),
        Err(err) => println!("Error parsing: {err}, {}", lexer.location.index)
    }
}