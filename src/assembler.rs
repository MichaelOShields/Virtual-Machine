use std::num::ParseIntError;
use std::fs;

/*
CORE IDEA:

[OPERATION] [MODE] [REGS] [OPTIONAL IMMEDIATE]
i.e.
move rr r0 r1; -> moves r1 to r0


FUNCTIONS:
fnc name:

(function stuff)

end



*/

#[derive(Debug)]
pub struct LexerError {
    message: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Int(u8),
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

}


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
        self.advance();

        let mut string_token: String = String::from("");

        while !self.peek().is_none() && self.peek() != Some('\r') { // not eof and not the end of the string

            
            string_token.push(self.peek().unwrap().to_ascii_lowercase());
            self.advance();
        }

        if self.peek().is_none() {

            return Err(LexerError{message: String::from("EOF before string end.")});

        }
        else {
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

        let parsed: Result<u8, ParseIntError> = int_token.parse::<u8>();

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

        self.skip_whitespace();

        if self.is_eof() {
            return Ok(Token::EOF);
        }
        else {

            let current_char: char = self.peek().unwrap();

            return match current_char {
                ';' => self.read_comment(),
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

    let mut lex: Lexer = Lexer::new(&code);

    let mut first = match lex.next_token() {
        Ok(t) => t,
        Err(e) => panic!("{}",(format!("Received LexerError {}", e.message))),
    };

    while first != Token::EOF {
        println!("{:?}", first);
        first = match lex.next_token() {
            Ok(t) => t,
            Err(e) => panic!("{}",(format!("Received LexerError {}", e.message))),
        };
    }
}



// Parser

pub enum MemExpr {
    Address ( u16 ),
    Reference ( String ),
}

pub enum DestExpr {
    Register { index: u8 },
    MemAddress ( MemExpr ),
}

pub enum Unsigned8Expr {
    Raw ( u8 ),
    Reference ( String ),
}

pub enum Unsigned16Expr {
    Raw ( u16 ),
    Reference ( String ),
}

pub enum SrcExpr {
    DestExpr ( DestExpr ),
    Unsigned8 ( Unsigned8Expr ),

}


pub enum Stmt {
    Move { dest: DestExpr, src: SrcExpr, length: u8, }
}