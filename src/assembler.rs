use std::num::ParseIntError;
use std::{fs, string};
use std::collections::HashMap;


/*
CORE IDEA:

[OPERATION] [MODE] [REGS] [OPTIONAL IMMEDIATE]
i.e.
move rr r0 r1; -> moves r1 to r0


*/

#[derive(Debug)]
pub struct LexerError {
    message: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Int( i64 ),
    Hex ( String ),
    Binary ( String ),
    Str(String),
    Comment(String),
    PLUS,
    MINUS,
    COLON,
    ASTERISK,
    FSLASH,
    UNDERSCORE,
    LPAREN,
    EXCLAMATION,
    RPAREN,
    LESSTHAN,
    GREATERTHAN,
    LESSTHANEQ,
    GREATERTHANEQ,
    LSQBRACK,
    RSQBRACK,
    COMMA,
    EQUALS,
    LCURL,
    RCURL,
    EOF,
    PERIOD,
    NEWLINE,
    TAB,

}

#[derive(Debug, PartialEq)]
pub struct Lexer {
    src: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            src: input.chars().collect(),
            pos: 0,
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn peek(&self) -> Option<char> {
        if self.pos >= self.src.len() { // eof
            return None;
        }
        else { // next char
            return Some(self.src[self.pos]);
        }
    }


    fn is_eof(&self) -> bool {
        return self.pos >= self.src.len();
    }


    fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.src[self.pos].is_whitespace() {
            self.advance();
        }
    }


    fn basic_token(&mut self, token: Token) -> Token {
        self.advance();
        return token;
    }


    fn read_string(&mut self) -> Result<Token, LexerError> {
        self.advance();

        let mut string_token: String = String::from("");

        while !self.peek().is_none() && self.peek() != Some('"') { // not eof and not the end of the string

            
            string_token.push(self.peek().unwrap());
            self.advance();
        }

        if self.peek().is_none() {

            return Err(LexerError{message: String::from("EOF before string end.")});

        }
        else {
            self.advance();
            return Ok(Token::Str(string_token));


        }    
    }

    fn read_comment(&mut self) -> Result<Token, LexerError> {
        self.advance(); // ;

        let mut string_token: String = String::from("");

        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            string_token.push(c.to_ascii_lowercase());
            self.advance();
        }

        if self.peek().is_none() {

            return Err(LexerError{message: String::from("EOF before string end.")});

        }
        else {
            // println!("Comment: {}", string_token);
            // println!("Current char: {}", self.peek().unwrap());
            self.advance();
            return Ok(Token::Comment(string_token));


        }    
    }

    fn read_ident(&mut self) -> Result<Token, LexerError> {
        let mut ident_token: String = String::from("");

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident_token.push(c.to_ascii_lowercase());
                self.advance();
            }
            else {
                break;
            }
        }
        // println!("Ident token: {:?}", ident_token);
        return Ok(Token::Ident(ident_token));
    }

    fn read_hex(&mut self) -> Result<Token, LexerError> {
        self.advance(); // 0
        self.advance(); // x
        return Ok(Token::Hex(match self.read_ident()? {
            Token::Ident(s) => s,
            _ => return Err(LexerError { message: "Expected a string w/in ident".to_string() }),
        }));

    }

    fn read_binary(&mut self) -> Result<Token, LexerError> {
        self.advance(); // 0
        self.advance(); // b
        return Ok(Token::Binary(match self.read_ident()? {
            Token::Ident(s) => s,
            _ => return Err(LexerError { message: "Expected a string w/in ident".to_string() }),
        }));

    }

    fn read_int(&mut self) -> Result<Token, LexerError> {
        let mut int_token: String = String::from("");

        while let Some(c) = self.peek() {
            if c.is_numeric() {
                int_token.push(c);
                self.advance();
            }
            else {
                break;
            }
        }
            // let int: u8 = int_token.parse()?;

        let parsed: Result<i64, ParseIntError> = int_token.parse::<i64>();

        return match parsed {
            Ok(value) => Ok(Token::Int(value)),
            Err(_) => Err(LexerError { message: String::from("Failure to parse integer in character stream.") })
        };
    }

    fn read_ineq(&mut self, token: Token) -> Result<Token,LexerError> {
        self.advance();
        match token {
            Token::GREATERTHAN => {
                // self.advance();
                match self.peek_next_token()? {
                    Token::EQUALS => {self.advance(); self.advance(); return Ok(Token::GREATERTHANEQ)},
                    _ => {self.advance(); return Ok(Token::GREATERTHAN);},
                }
            },
            Token::LESSTHAN => {
                self.advance();
                match self.peek_next_token()? {
                    Token::EQUALS => {self.advance(); self.advance(); return Ok(Token::LESSTHANEQ)},
                    _ => {return Ok(Token::LESSTHAN);},
                }
            },
            _ => return Err(LexerError { message: "Received unexpected ineq token".to_string() }),
        }
    }
    


    pub fn next_token(&mut self) -> Result<Token,LexerError> {

        // self.skip_whitespace();

        if self.is_eof() {
            return Ok(Token::EOF);
        }
        else {

            let current_char: char = self.peek().unwrap();
            // println!("{}", current_char);

            return match current_char {
                ';' => self.read_comment(),
                '\r' => match self.peek_next_char()? {
                    '\n' => {self.advance(); return Ok(self.basic_token(Token::NEWLINE))},
                    '\t' => {self.advance(); return Ok(self.basic_token(Token::TAB))},
                    _ => return Err(LexerError { message: "Unexpected \\r character.".to_string() }),
                },
                '\n' => Ok(self.basic_token(Token::NEWLINE)),
                '\t' => Ok(self.basic_token(Token::TAB)),
                '+' => Ok(self.basic_token(Token::PLUS)),
                '-' => Ok(self.basic_token(Token::MINUS)),
                '_' => Ok(self.basic_token(Token::UNDERSCORE)),
                '(' => Ok(self.basic_token(Token::LPAREN)),
                ')' => Ok(self.basic_token(Token::RPAREN)),
                '[' => Ok(self.basic_token(Token::LSQBRACK)),
                ']' => Ok(self.basic_token(Token::RSQBRACK)),
                '/' => Ok(self.basic_token(Token::FSLASH)),
                ':' => Ok(self.basic_token(Token::COLON)),
                '{' => Ok(self.basic_token(Token::LCURL)),
                '}' => Ok(self.basic_token(Token::RCURL)),
                '*' => Ok(self.basic_token(Token::ASTERISK)),
                '=' => Ok(self.basic_token(Token::EQUALS)),
                ',' => Ok(self.basic_token(Token::COMMA)),
                '<' => Ok(self.read_ineq(Token::LESSTHAN)?),
                '>' => Ok(self.read_ineq(Token::GREATERTHAN)?),
                '!' => Ok(self.basic_token(Token::EXCLAMATION)),
                '.' => Ok(self.basic_token(Token::PERIOD)),
                '~' => self.read_comment(),
                '"' => self.read_string(),
                c if c.is_alphabetic() => self.read_ident(),
                c if c.is_numeric() => match c {
                    '0' => match self.peek_next_char()? {
                        'x' => self.read_hex(),
                        'b' => self.read_binary(),
                        _ => self.read_int(),
                    },
                    _ => self.read_int(),
                },
                c if c.is_ascii_whitespace() => match c {
                    // '\t' => Ok(self.basic_token(Token::TAB)),
                    _ => {self.advance(); return self.next_token();},
                },

                _ => Err(LexerError{message: format!("Couldn't read character {}", current_char)}),
            };
            


        }
    }

    pub fn peek_next_char(&mut self) -> Result<char, LexerError> {
        let current_pos: usize = self.pos;
        self.pos += 1;
        let character = match self.peek() {
            Some(c) => c,
            None => return Err(LexerError { message: "Expected character but got none".to_string() }),
        };

        // println!("{}", character);
        
        self.pos = current_pos;
        return Ok(character);
    }


    pub fn peek_next_token(&mut self) -> Result<Token, LexerError> {
        let current_pos: usize = self.pos;
        let token= self.next_token();
        
        self.pos = current_pos;
        return token;

    }

}





pub fn assem(path: String) {
    let code = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {println!("Couldn't read file: {:?}", e); return;}
    };

    let lex: Lexer = Lexer::new(&code);

    let mut parser = Parser::new(lex);

    let mut first = match parser.next_stmt() {
            Ok(s) => s,
            Err(e) => panic!("Got PE {:?}", e),
        };
    // println!("First: {:?}", first);

    while first != Stmt::End {
        println!("Stmt: {:?}", first);
        first = match parser.next_stmt() {
            Ok(s) => s,
            Err(e) => {println!("Error: {:?}", e); break;}
        };
    }

    // let mut first = match lex.next_token() {
    //     Ok(t) => t,
    //     Err(e) => panic!("{}",(format!("Received LexerError {}", e.message))),
    // };

    // while first != Token::EOF {
    //     println!("{:?}", first);
    //     first = match lex.next_token() {
    //         Ok(t) => t,
    //         Err(e) => panic!("{}",(format!("Received LexerError {}", e.message))),
    //     };
    // }
    // println!("{:?}", first);
}



// Parser


#[derive(Debug, Clone, PartialEq)]
pub enum DoubleMode {
    Rm,
    Ri,
    Mr,
    Mi,
    Rr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SingleMode {
    R,
    I,
    M,
}




#[derive(Debug, PartialEq)]
pub enum StrExpr {
    Raw ( String ),
    Reference ( String ),
}
#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Plus,
    Minus,
    BitNot,
}
#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
}
#[derive(Debug, PartialEq)]
pub enum NumExpr {
    Raw ( i64 ),
    Reference ( String ),
}
#[derive(Debug, PartialEq)]
pub enum Operand {
    Register ( u8 ) ,
    Immediate ( Expr ),
}
#[derive(Debug, PartialEq)]
pub enum Expr {
    Str ( StrExpr ),
    Num ( NumExpr ),
    Operand ( Box<Expr> ),
    BinaryExpr {
        a: Box<Expr>,
        operation: BinaryOp,
        b: Box<Expr>,
    },
    UnaryExpr {
        operation: UnaryOp,
        operatee: Box<Expr>,
    }

}


#[derive(Debug, PartialEq)]
pub enum Stmt {
    DoubleOperation {
        opid: String, // opcode 3 letter abbr
        mode: DoubleMode,
        dest: Operand,
        src: Operand,
    },
    SingleOperation {
        opid: String,
        mode: SingleMode,
        operand: Operand,
    },
    Signal {
        name: String,
        args: Vec<Expr>,

    },
    Label ( String ),
    Newline,
    Comment ( String ),
    End,
}

#[derive(Debug, PartialEq)]
pub struct ParserError {
    message: String,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Parser {
    parser_lexer: Lexer,
    current_token: Token,
    static_modes: HashMap<char, SingleMode>,
    double_modes: HashMap<String, DoubleMode>,
}

impl Parser {
    pub fn new(mut passed_lex: Lexer) -> Self {
        let first = passed_lex.next_token().unwrap();
        let mut static_modes: HashMap<char, SingleMode> = HashMap::new();
        static_modes.insert('r', SingleMode::R);
        static_modes.insert('m', SingleMode::M);
        static_modes.insert('i', SingleMode::I);

        let mut double_modes: HashMap<String, DoubleMode> = HashMap::new();
        double_modes.insert("rm".to_string(), DoubleMode::Rm);
        double_modes.insert("rr".to_string(), DoubleMode::Rr);
        double_modes.insert("ri".to_string(), DoubleMode::Ri);
        double_modes.insert("mr".to_string(), DoubleMode::Mr);
        double_modes.insert("mi".to_string(), DoubleMode::Mi);
        Self {
            parser_lexer: passed_lex,
            current_token: first,
            static_modes,
            double_modes,
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParserError> {
        let val = match &self.current_token {
            Token::Ident(n) => n.clone(),
            _ => return Err(ParserError { message: format!("Expected identifier but got token type {:?}", self.current_token) }),
        };

        self.advance()?;
        return Ok(val);
    }

    fn parse_register(name: &str) -> Option<u8> {
        if name.len() < 2 { return None; }
        if !name.starts_with('r') { return None; }

        name[1..].parse::<u8>().ok()
    }


    fn expect_operand(&mut self) -> Result<Operand, ParserError> {
        match &self.current_token {
            Token::Ident(name) => { // Try register, otherwise is immediate reference
                if let Some(reg) = Self::parse_register(name) {
                    self.advance()?;
                    return Ok(Operand::Register(reg));
                }
                else {
                    let expr = Expr::Num(NumExpr::Reference(name.clone()));
                    self.advance()?;
                    return Ok(Operand::Immediate(expr));
                }
            },
            Token::Int(val) => { // Immediate value
                let expr = Expr::Num(NumExpr::Raw(*val));
                self.advance()?;
                return Ok(Operand::Immediate(expr));
            },
            _ => return Err(ParserError { message: format!("Couldn't understand operand expr {:?}", self.current_token) }),

        };
    }

    fn parse_double_op(&mut self, opid: String) -> Result<Stmt, ParserError> {
        match self.expect_ident() {
            Ok(s) => match s {
                _c if s == "mov" => println!("Correct mov identifier"),
                _ => return Err(ParserError { message: "Expected identifier 'mov'".to_string() })
            },
            Err(e) => return Err(e),
        };
        let mode_ident = self.expect_ident()?;
        println!("mode: {:?}", mode_ident);
        let mode = match self.double_modes.get(&mode_ident) {
            Some(m) => m,
            None => return Err(ParserError { message: "Unable to get mode".to_string() }),
        }.clone();
        let dest = self.expect_operand()?;
        let src = self.expect_operand()?;
        return Ok(Stmt::DoubleOperation { opid, mode, dest, src })
    }

    fn basic_token(&mut self, expected: Token) -> Result<(), ParserError> {
        match &self.current_token {
            t if t == &expected => (),
            _ => return Err(ParserError { message: format!("Expected token of type {:?} but got token <{:?}>", expected, self.current_token)}),
        };

        self.advance()?;
        return Ok(());
    }

    fn basic_stmt(&mut self, expected_stmt: Stmt, expected_token: Token) -> Result<Stmt, ParserError> {
        self.basic_token(expected_token)?;
        Ok(expected_stmt)

    }

    fn parse_org(&mut self) -> Result<Stmt, ParserError> {
        let addr: Expr = match &self.current_token {
            Token::Ident(s) => Expr::Num(NumExpr::Reference(s.to_string())),
            Token::Int(i) => Expr::Num(NumExpr::Raw(*i)),
            _ => return Err(ParserError { message: format!("Expected NumExpr, got {:?}", self.current_token) }),
        };
        self.advance()?;
        return Ok(Stmt::Signal { name: "org".to_string(), args: vec![addr] });
    }

    fn advance(&mut self) -> Result<(), ParserError> {
        let tok = self.parser_lexer.next_token()
            .map_err(|e| ParserError {
                message: format!("LexerError {:?}", e),
            })?;
        self.current_token = tok;
        Ok(())
    }


    fn parse_signal(&mut self) -> Result<Stmt, ParserError> {
        self.basic_token(Token::PERIOD)?;
        let sigid = match &self.current_token {
            Token::Ident(s) => s.clone(),
            _ => return Err(ParserError { message: format!("Expected signal identifier, got {:?}", self.current_token) })
        };
        self.advance()?;
        return Ok(match sigid.as_str() {
            "org" => self.parse_org()?,
            _ => return Err(ParserError { message: format!("Couldn't parse signal {:?}", sigid) }),
        });
    }

    pub fn next_stmt(&mut self) -> Result<Stmt, ParserError> {
        // println!("Looking at token {:?}", self.current_token);
        if self.current_token == Token::EOF {
            return Ok(Stmt::End);
        }
        else if let Token::Comment(ref c) = self.current_token {
            let name = c.clone();
            self.advance()?;
            return Ok(Stmt::Comment(name));
        }
        else if let Token::NEWLINE = self.current_token {
            self.basic_token(Token::NEWLINE)?;
            return Ok(Stmt::Newline);
        }
        else if let Token::Ident(ref s) = self.current_token {
            if s == "mov" || s == "add" {
                return Ok(self.parse_double_op(s.to_string())?);
            }
            else {
                if matches!(self.parser_lexer.peek_next_token().unwrap(), Token::COLON) {
                    let name = s.clone();
                    self.advance()?;
                    self.basic_token(Token::COLON)?;
                    return Ok(Stmt::Label(name));
                }
                else {
                    return Err(ParserError { message: format!("Couldn't identify ident {:?}",s   ) });
                }
            }
        }
        else if let Token::PERIOD = self.current_token {
            return Ok(self.parse_signal()?);
        }
        else {
            return Err(ParserError { message: format!("Couldn't parse token {:?}", self.current_token) });
        }
    }
}