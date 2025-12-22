use std::num::ParseIntError;
use std::{fs, string};
use std::collections::HashMap;

use crate::binary::{get_bits_lsb, get_bits_msb};


/*
CORE IDEA:

[OPERATION] [MODE] [REGS] [OPTIONAL IMMEDIATE]
i.e.
move rr r0, r1; -> moves r1 to r0

Returns Vec<Position, Instruction>

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
    Char( char ),
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

    fn read_char(&mut self) -> Result<Token, LexerError> {
        self.advance();

        if !self.peek().is_none() && self.peek() != Some('\'') {
            let char_token = self.peek().unwrap();
            self.advance(); // char
            self.advance(); // '

            return Ok(Token::Char( char_token ));
        }
        else {
            return Err(LexerError{message: String::from("Couldn't lex character")});
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

    fn read_ident(&mut self, ignore_underscores: bool) -> Result<Token, LexerError> {
        let mut ident_token: String = String::from("");

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                if c == '_' {
                    if !ignore_underscores {
                        ident_token.push(c.to_ascii_lowercase());
                        self.advance();
                    }
                    else {
                        self.advance();
                    }
                }
                else {
                    ident_token.push(c.to_ascii_lowercase());
                    self.advance();
                }
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
        return Ok(Token::Hex(match self.read_ident(true)? {
            Token::Ident(s) => s,
            _ => return Err(LexerError { message: "Expected a string w/in ident".to_string() }),
        }));

    }

    fn read_binary(&mut self) -> Result<Token, LexerError> {
        self.advance(); // 0
        self.advance(); // b
        return Ok(Token::Binary(match self.read_ident(true)? {
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
                    _ => return Err(LexerError { message: "Unexpected /r character.".to_string() }),
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
                '\'' => self.read_char(),
                c if c.is_alphabetic() => self.read_ident(false),
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





// Parser


#[derive(Debug, Clone, PartialEq)]
pub enum DoubleMode {
    Rm,
    Ri,
    Mr,
    Rr,
    Rmi, // taking a mem addr, getting its val from mem, putting it into reg
    Mir, // taking a register, putting its val into mem via an immediate val
}

#[derive(Debug, Clone, PartialEq)]
pub enum SingleMode {
    R,
    I,
    M,
    Mi, // memory from immediate operand
}




#[derive(Debug, PartialEq, Clone)]
pub enum StrExpr {
    Raw ( String ),
    // Reference ( String ),
}
#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Plus,
    Minus,
    BitNot,
}
#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Div,
    Mul,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Function {
    Hi ( NumExpr ),  // high byte of mem addr
    Lo ( NumExpr ), // low byte of mem addr
}

#[derive(Debug, PartialEq, Clone)]
pub enum NumExpr {
    Raw ( i64 ),
    Reference ( String ),
    Function ( Box<Function> ),
    BinaryOperation {
        a: Box<NumExpr>,
        operand: BinaryOp,
        b: Box<NumExpr>,
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Register ( u8 ) ,
    Immediate ( Expr ),
}
#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Str ( StrExpr ),
    Num ( NumExpr ),
    Operand ( Box<Expr> ),
    UnaryExpr {
        operation: UnaryOp,
        operatee: Box<Expr>,
    }

}

#[derive(Debug, PartialEq, Clone)]
pub enum OpKind {
    Double, // takes 2 operands
    Single, // takes 1 operand
    Zero, // takes 0 operands
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperandLength {
    Unsigned16,
    Unsigned8,
    Any,
    Zero,
}


#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    DoubleOperation {
        opid: String, // opcode 3 letter abbr
        mode: DoubleMode,
        dest: Operand,
        src: Operand,
        operand_length: OperandLength,
    },
    SingleOperation {
        opid: String,
        mode: SingleMode,
        operand: Operand,
        operand_length: OperandLength,
    },
    ZeroOperation {
        opid: String,
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
    single_modes: HashMap<char, SingleMode>,
    double_modes: HashMap<String, DoubleMode>,
    ops: HashMap<String, (OpKind, OperandLength)>,
}

impl Parser {
    pub fn new(mut passed_lex: Lexer) -> Self {
        let first = passed_lex.next_token().unwrap();
        let mut single_modes: HashMap<char, SingleMode> = HashMap::new();
        single_modes.insert('r', SingleMode::R);
        single_modes.insert('m', SingleMode::M);
        single_modes.insert('i', SingleMode::I);

        let mut double_modes: HashMap<String, DoubleMode> = HashMap::new();
        double_modes.insert("rm".to_string(), DoubleMode::Rm);
        double_modes.insert("rr".to_string(), DoubleMode::Rr);
        double_modes.insert("ri".to_string(), DoubleMode::Ri);
        double_modes.insert("mr".to_string(), DoubleMode::Mr);

        let mut ops: HashMap<String, (OpKind, OperandLength)> = HashMap::new();

        ops.insert("nop".to_string(), (OpKind::Zero, OperandLength::Zero));
        ops.insert("ret".to_string(), (OpKind::Zero, OperandLength::Zero));
        ops.insert("hlt".to_string(), (OpKind::Zero, OperandLength::Zero));

        ops.insert("kret".to_string(), (OpKind::Zero, OperandLength::Zero));

        ops.insert("sys".to_string(), (OpKind::Zero, OperandLength::Zero));

        ops.insert("pnk".to_string(), (OpKind::Zero, OperandLength::Zero));

    

        ops.insert("jmp".to_string(), (OpKind::Single, OperandLength::Unsigned16));
        ops.insert("jz".to_string(),  (OpKind::Single, OperandLength::Unsigned16));
        ops.insert("jc".to_string(),  (OpKind::Single, OperandLength::Unsigned16));
        ops.insert("jo".to_string(),  (OpKind::Single, OperandLength::Unsigned16));
        ops.insert("js".to_string(),  (OpKind::Single, OperandLength::Unsigned16));
        ops.insert("jnz".to_string(), (OpKind::Single, OperandLength::Unsigned16));
        ops.insert("jg".to_string(),  (OpKind::Single, OperandLength::Unsigned16));
        ops.insert("jl".to_string(),  (OpKind::Single, OperandLength::Unsigned16));

        ops.insert("gsp".to_string(), (OpKind::Single, OperandLength::Unsigned16));

        ops.insert("call".to_string(), (OpKind::Single, OperandLength::Unsigned16));

        
        ops.insert("push".to_string(), (OpKind::Single, OperandLength::Unsigned8));
        ops.insert("pop".to_string(),  (OpKind::Single, OperandLength::Unsigned8));

        ops.insert("dbg".to_string(), (OpKind::Single, OperandLength::Unsigned8));

        
        ops.insert("shl".to_string(), (OpKind::Single, OperandLength::Unsigned8));
        ops.insert("sar".to_string(), (OpKind::Single, OperandLength::Unsigned8));

        
        ops.insert("ssp".to_string(),  (OpKind::Single, OperandLength::Unsigned16)); // stack pointer set
        ops.insert("skip".to_string(), (OpKind::Single, OperandLength::Unsigned8));  // skip N instructions/bytes

        
        ops.insert("mov".to_string(), (OpKind::Double, OperandLength::Unsigned8));

        ops.insert("shr".to_string(), (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("shrw".to_string(), (OpKind::Double, OperandLength::Unsigned8));

        
        ops.insert("add".to_string(), (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("sub".to_string(), (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("mul".to_string(), (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("div".to_string(), (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("mod".to_string(), (OpKind::Double, OperandLength::Unsigned8));

        
        ops.insert("and".to_string(), (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("or".to_string(),  (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("xor".to_string(), (OpKind::Double, OperandLength::Unsigned8));
        ops.insert("not".to_string(), (OpKind::Single, OperandLength::Unsigned8));

        
        ops.insert("cmp".to_string(), (OpKind::Double, OperandLength::Unsigned8));



        Self {
            parser_lexer: passed_lex,
            current_token: first,
            single_modes,
            double_modes,
            ops,
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

    fn function_from_ident(&mut self, ident: &str) -> Result<Function, ParserError> {
        return Ok(match ident {
            "hi" => {
                let expr = self.expect_numexpr()?;
                self.basic_token(Token::RPAREN)?;
                Function::Hi(expr)
            },
            "lo" => {
                let expr = self.expect_numexpr()?;
                self.basic_token(Token::RPAREN)?;
                Function::Lo(expr)
            },
            _ => return Err(ParserError { message: "Couldn't understand function".to_string() }),
        })
    }

    fn expect_numexpr(&mut self) -> Result<NumExpr, ParserError> {
        return Ok(match &self.current_token {
            Token::Ident(name) => { // Try register, otherwise is immediate reference

                let expr = NumExpr::Reference(name.clone());
                self.advance()?;
                expr
            },
            Token::Int(val) => { // Immediate value
                let expr = NumExpr::Raw(*val);
                self.advance()?;
                expr
            },
            Token::Hex(h) => {
                let expr = NumExpr::Raw(match i64::from_str_radix(h, 16) {
                    Ok(i) => i,
                    Err(e) => return Err(ParserError { message: format!("Got hex error {:?}", e) }),
                });
                self.advance()?;
                expr
            },
            Token::Binary(h) => {
                let expr = NumExpr::Raw(match i64::from_str_radix(h, 2) {
                    Ok(i) => i,
                    Err(e) => return Err(ParserError { message: format!("Got binary error {:?}", e) }),
                });
                self.advance()?;
                expr
            },
            Token::LPAREN => {
                self.basic_token(Token::LPAREN)?;
                let a = self.expect_numexpr()?;
                let next_tok: Token = self.peek_token()?;
                let op: BinaryOp = match next_tok {
                    Token::ASTERISK => BinaryOp::Mul,
                    Token::PLUS => BinaryOp::Add,
                    Token::MINUS => BinaryOp::Sub,
                    Token::FSLASH => BinaryOp::Div,
                    _ => return Err(ParserError { message: format!("Expected operation exp but got {:?}", self.peek_token()?) }),
                };
                self.basic_token(next_tok)?;

                let b = self.expect_numexpr()?;
                self.basic_token(Token::RPAREN)?;
                NumExpr::BinaryOperation { a: Box::from(a), operand: op, b: Box::from(b) }
            }
            _ => return Err(ParserError { message: format!("Couldn't understand numexpr {:?}", self.current_token) }),
        });
    }


    fn expect_operand(&mut self) -> Result<Operand, ParserError> {
        match &self.current_token {
            Token::Ident(name) => { // Try register, otherwise is immediate reference

                if let Some(reg) = Self::parse_register(name) {
                    self.advance()?;
                    return Ok(Operand::Register(reg));
                }
                else if matches!(self.parser_lexer.peek_next_token().unwrap(), Token::LPAREN) {
                    let name = name.clone();
                    self.advance()?;
                    self.basic_token(Token::LPAREN)?;

                    let func = self.function_from_ident(&name)?;

                    return Ok(Operand::Immediate(Expr::Num(NumExpr::Function(Box::from(func)))));



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
            Token::Hex(h) => {
                let expr = Expr::Num(NumExpr::Raw(match i64::from_str_radix(h, 16) {
                    Ok(i) => i,
                    Err(e) => return Err(ParserError { message: format!("Got hex error {:?}", e) }),
                }));
                self.advance()?;
                return Ok(Operand::Immediate(expr));
            },
            Token::Binary(h) => {
                let expr = Expr::Num(NumExpr::Raw(match i64::from_str_radix(h, 2) {
                    Ok(i) => i,
                    Err(e) => return Err(ParserError { message: format!("Got binary error {:?}", e) }),
                }));
                self.advance()?;
                return Ok(Operand::Immediate(expr));
            },
            /*
            Token::Ident(s) => Expr::Num(NumExpr::Reference(s.to_string())),
            Token::Int(i) => Expr::Num(NumExpr::Raw(*i)),
            Token::Hex(h) => Expr::Num(NumExpr::Raw(match i64::from_str_radix(h, 16) {
                Ok(i) => i,
                Err(e) => return Err(ParserError { message: format!("Got hex error {:?}", e) }),
            })),
            Token::Binary(h) => Expr::Num(NumExpr::Raw(match i64::from_str_radix(h, 2) {
                Ok(i) => i,
                Err(e) => return Err(ParserError { message: format!("Got binary error {:?}", e) }),
            })),
            _ => return Err(ParserError { message: format!("Expected NumExpr, got {:?}", self.current_token) }),
             */
            _ => return Err(ParserError { message: format!("Couldn't understand operand expr {:?}", self.current_token) }),

        };
    }

    fn parse_double_op(&mut self, opid: String, operand_length: OperandLength) -> Result<Stmt, ParserError> {
        self.expect_ident()?; // opid
        let mode_ident = self.expect_ident()?;
        // println!("mode: {:?}", mode_ident);
        let mut mode = match self.double_modes.get(&mode_ident) {
            Some(m) => m,
            None => return Err(ParserError { message: format!("Unable to get mode {:?}", mode_ident) }),
        }.clone();

        let dest = self.expect_operand()
            .map_err(|_| ParserError { message: "Expected dest operand".into() })?;

        self.basic_token(Token::COMMA)?;
        let src = self.expect_operand()
            .map_err(|_| ParserError { message: "Expected src operand".into() })?;

        if let Operand::Register(_) = src {
            ()
        }
        else if mode == DoubleMode::Rm {
            mode = DoubleMode::Rmi;
        }
        if let Operand::Register(_) = dest {
            ()
        }
        else if mode == DoubleMode::Mr {
            mode = DoubleMode::Mir;
        }

        return Ok(Stmt::DoubleOperation { opid, mode, dest, src, operand_length })
    }

    fn parse_single_op(&mut self, opid: String, operand_length: OperandLength) -> Result<Stmt, ParserError> {
        self.expect_ident()?; // opid
        let mode_ident = self.expect_ident()?;
        // println!("mode: {:?}", mode_ident);
        let mut mode = match self.single_modes.get(match (&mode_ident).as_str() {
            "r" => &'r',
            "i" => &'i',
            "m" => &'m',
            _ => return Err(ParserError { message: format!("Received unexpected mode {:?}", mode_ident) }),
        }) {
            Some(m) => m,
            None => return Err(ParserError { message: "Unable to get mode".to_string() }),
        }.clone();
        let operand = self.expect_operand()?;

        if let Operand::Register(_) = operand {
            ()
        }
        else if mode == SingleMode::M {
            mode = SingleMode::Mi;
        }

        return Ok(Stmt::SingleOperation { opid, mode, operand, operand_length })
    }

    fn parse_zero_op(&mut self, opid: String) -> Result<Stmt, ParserError> {
        self.expect_ident()?; // opid
        return Ok(Stmt::ZeroOperation { opid })
    }

    fn basic_token(&mut self, expected: Token) -> Result<(), ParserError> {
        match &self.current_token {
            t if t == &expected => (),
            _ => return Err(ParserError { message: format!("Expected token of type {:?} but got token <{:?}>", expected, self.current_token)}),
        };

        self.advance()?;
        return Ok(());
    }

    fn advance(&mut self) -> Result<(), ParserError> {
        let tok = self.parser_lexer.next_token()
            .map_err(|e| ParserError {
                message: format!("LexerError {:?}", e),
            })?;
        self.current_token = tok;
        Ok(())
    }

    fn taking_next_args(&mut self) -> Result<bool, ParserError> {
        return Ok(match self.current_token {
            
            Token::EOF => false,
            Token::Comment(_) => false,
            Token::NEWLINE => false,
            _ => true,
        });
    }



    fn parse_sig(&mut self, sigid: String) -> Result<Stmt, ParserError> {
        let mut args: Vec<Expr> = vec![];
        let taking_args = self.taking_next_args()?;

        while taking_args {
            let vectok: Expr = match &self.current_token {
                Token::Ident(s) => Expr::Num(NumExpr::Reference(s.to_string())),
                Token::Int(i) => Expr::Num(NumExpr::Raw(*i)),
                Token::Hex(h) => Expr::Num(NumExpr::Raw(match i64::from_str_radix(h, 16) {
                    Ok(i) => i,
                    Err(e) => return Err(ParserError { message: format!("Got hex error {:?}", e) }),
                })),
                Token::Binary(h) => Expr::Num(NumExpr::Raw(match i64::from_str_radix(h, 2) {
                    Ok(i) => i,
                    Err(e) => return Err(ParserError { message: format!("Got binary error {:?}", e) }),
                })),
                Token::Str(s) => Expr::Str(StrExpr::Raw(s.to_string())),
                Token::Char(c) => Expr::Num(NumExpr::Raw(*c as i64)),
                _ => return Err(ParserError { message: format!("Expected Expr, got {:?}", self.current_token) }),
            };
            self.advance()?;
            args.push(vectok);
            match self.basic_token(Token::COMMA) {
                Ok(()) => (),
                Err(_e) => break,
            }
        };
        
        return Ok(Stmt::Signal { name: sigid, args });
    }


    fn peek_token(&mut self) -> Result<Token, ParserError> {
        return Ok(match self.parser_lexer.peek_next_token() {
            Ok(t) => t,
            Err(e) => return Err(ParserError { message: format!("Got LexerError {:?}", e) }),
        })
    }

    fn parse_signal(&mut self) -> Result<Stmt, ParserError> {
        self.basic_token(Token::PERIOD)?;
        let sigid = match &self.current_token {
            Token::Ident(s) => s.clone(),
            _ => return Err(ParserError { message: format!("Expected signal identifier, got {:?}", self.current_token) })
        };
        self.advance()?; // sig ident
        return Ok(self.parse_sig(sigid.to_string())?);
    }

    fn next_stmt(&mut self) -> Result<Stmt, ParserError> {
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
            match self.ops.get(s.as_str()) {
                Some(o) => {
                    let (operation_type, operand_length) = o;
                    match operation_type {
                        OpKind::Double => return Ok(self.parse_double_op(s.clone(), operand_length.clone())?),
                        OpKind::Single => return Ok(self.parse_single_op(s.clone(), operand_length.clone())?),
                        OpKind::Zero => return Ok(self.parse_zero_op(s.clone())?),
                    };
                },
                None => (),
            };
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
        else if let Token::PERIOD = self.current_token {
            return Ok(self.parse_signal()?);
        }
        else {
            return Err(ParserError { message: format!("Couldn't parse token {:?}", self.current_token) });
        }
    }


    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParserError> {
        let mut program: Vec<Stmt> = vec![];

        loop {
            let stmt = self.next_stmt()?;
            if stmt == Stmt::End {
                break;
            };
            program.push(stmt);
        }
        Ok(program)
    }
}


// assembler will reserve R6 and R7 for under the hood memory ops

#[derive(PartialEq, Debug, Clone)]
enum AssembleMode {
    CountBytes,
    Assemble,
}

#[derive(Debug)]
pub struct AssemblerError {
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub struct Assembler {
    pc: u16,
    current_pos: u16,
    output: Vec<u8>,
    labels: HashMap<String, u16>,
    consts: HashMap<String, u8>,
    program: Vec<Stmt>,
    mode: AssembleMode,
}

impl Assembler {
    pub fn new(program: Vec<Stmt>) -> Self {
        Self {
            pc: 0_u16,
            current_pos: 0_u16,
            output: vec![],
            labels: HashMap::new(),
            consts: HashMap::new(),
            program,
            mode: AssembleMode::CountBytes,
        }
    }

    fn opcode_from_opid(&mut self, opid: String) -> Result<u8, AssemblerError> {
        return Ok(match opid.as_str() {
            "nop"  => 0b000_000,
            "mov"  => 0b000_001,
            "add"  => 0b000_010,
            "sub"  => 0b000_011,
            "mul"  => 0b000_100,
            "div"  => 0b000_101,
            "mod"  => 0b000_110,
            "and"  => 0b000_111,
            "or"   => 0b001_000,
            "xor"  => 0b001_001,
            "not"  => 0b001_010,
            "jmp"  => 0b001_011,
            "jz"   => 0b001_100,
            "jc"   => 0b001_101,
            "jo"   => 0b001_110,
            "js"   => 0b001_111,
            "jnz"  => 0b010_000,
            "jg"   => 0b010_001,
            "jl"   => 0b010_010,
            "cmp"  => 0b010_011,
            "push" => 0b010_100,
            "pop"  => 0b010_101,
            "call" => 0b010_110,
            "ret"  => 0b010_111,
            "shl"  => 0b011_000,
            "shr"  => 0b011_001,
            "sar"  => 0b011_010,
            "ssp"  => 0b011_011, // set stack pointer
            "skip" => 0b011_100,
            "sys" => 0b011_101,
            "kret" => 0b011_110,
            "gsp" => 0b011_111,
            "pnk" => 0b100_000,
            "dbg" => 0b100_001,
            "shrw" => 0b100_010,
            "hlt"  => 0b111_111,
            _ => return Err(AssemblerError { message: "Unable to parse opcode".to_string() }),
        });
    }

    fn dbmode_convert(&mut self, mode: DoubleMode) -> Result<u8, AssemblerError> {
        return Ok(match mode {
            DoubleMode::Rr => 0b0000,
            DoubleMode::Rm => 0b0001,
            DoubleMode::Mr => 0b0010,
            DoubleMode::Ri => 0b0011,
            DoubleMode::Rmi => 0b0100,
            DoubleMode::Mir => 0b0101,
            _ => return Err(AssemblerError { message: "Unable to parse mode".to_string() }),
        });
    }

    fn smode_convert(&mut self, mode: SingleMode) -> Result<u8, AssemblerError> {
        return Ok(match mode {
            SingleMode::R => 0b0000,
            SingleMode::M => 0b0001,
            SingleMode::I => 0b0010,
            SingleMode::Mi => 0b0011,
            _ => return Err(AssemblerError { message: "Unable to parse mode".to_string() }),
        });
    }

    fn register_from_operand(&mut self, op: Operand) -> Result<u8, AssemblerError> {
        return Ok(match op {
            Operand::Register(num) => num,
            _ => return Err(AssemblerError { message: format!("Expected register, got {:?}", op) })
        });
    }

    fn get_addr_from_numexpr(&mut self, numexpr: NumExpr) -> Result<(u8, u8), AssemblerError> {
        let mem_addr = match numexpr {
            NumExpr::Reference(r) => self.get_label(r.as_str())?,
            NumExpr::Raw(i) => i as u16,
            NumExpr::Function(_) => return Err(AssemblerError { message: "Function received when getting address".to_string() }),
            NumExpr::BinaryOperation { a, operand, b } => self.eval_num_bin_op(*a, operand, *b)? as u16,

        };
        let m1: u8 = get_bits_msb(mem_addr, 0, 7) as u8;
        let m2: u8 = get_bits_msb(mem_addr, 7, 15) as u8;
        return Ok((m1, m2));
    }

    // fn mem_to_reg_from_operand(&mut self, mem: Operand) -> Result<(u8, Vec<u8>, Vec<u8>), AssemblerError> { // result is register to grab mem addr, first vec<u8> is just loading the mem addr into R6 and R7, second vec<u8> is restoring R6 and R7
    //     let reg: u8 = 6;
    //     let mut setup: Vec<u8> = vec![];
    //     let mut cleanup: Vec<u8> = vec![];
    //     match mem {
    //         Operand::Immediate(e) => match e {
    //             Expr::Num(numex) => {
    //                 let savereg6: Stmt = Stmt::SingleOperation { opid: "push".to_string(), mode: SingleMode::R, operand: Operand::Register(6), operand_length: OperandLength::Unsigned8 };
    //                 let savereg7: Stmt = Stmt::SingleOperation { opid: "push".to_string(), mode: SingleMode::R, operand: Operand::Register(7), operand_length: OperandLength::Unsigned8 };
    //                 let mut loadingreg6 = self.assemble_single_op(savereg6)?;
    //                 let mut loadingreg7 = self.assemble_single_op(savereg7)?;
    //                 setup.append(&mut loadingreg6);
    //                 setup.append(&mut loadingreg7);
    //                 let (m1, m2) = self.get_addr_from_numexpr(numex)?;
    //                 let movem1toreg6stmt = Stmt::DoubleOperation { opid: "mov".to_string(), mode: DoubleMode::Ri, dest: Operand::Register(6), src: Operand::Immediate(Expr::Num(NumExpr::Raw(m1 as i64))), operand_length: OperandLength::Unsigned8 };
    //                 let movem2toreg7stmt = Stmt::DoubleOperation { opid: "mov".to_string(), mode: DoubleMode::Ri, dest: Operand::Register(7), src: Operand::Immediate(Expr::Num(NumExpr::Raw(m2 as i64))), operand_length: OperandLength::Unsigned8 };
    //                 let mut loadingm1 = self.assemble_double_op(movem1toreg6stmt)?;
    //                 let mut loadingm2 = self.assemble_double_op(movem2toreg7stmt)?;
    //                 setup.append(&mut loadingm1);
    //                 setup.append(&mut loadingm2);

    //                 let loadreg6: Stmt = Stmt::SingleOperation { opid: "pop".to_string(), mode: SingleMode::R, operand: Operand::Register(6), operand_length: OperandLength::Unsigned8 };
    //                 let loadreg7: Stmt = Stmt::SingleOperation { opid: "pop".to_string(), mode: SingleMode::R, operand: Operand::Register(7), operand_length: OperandLength::Unsigned8 };
    //                 let mut loadingpopreg6 = self.assemble_single_op(loadreg6)?;
    //                 let mut loadingpopreg7 = self.assemble_single_op(loadreg7)?;
    //                 cleanup.append(&mut loadingpopreg7);
    //                 cleanup.append(&mut loadingpopreg6);



    //                 return Ok((reg, setup, cleanup));
    //             },
    //             _ => return Err(AssemblerError { message: "Received unexpected immediate operand expression type.".to_string() })
    //         },
    //         Operand::Register(n) => {return Ok((n, vec![], vec![]))},
    //     };

    // }

    fn set_pc(&mut self, new_pc: u16) {
        self.pc = new_pc;
    }

    fn inc_pc(&mut self, amt: u16) {
        self.pc += amt;
    }

    fn assemble_zero_op(&mut self, opid: String) -> Result<Vec<u8>, AssemblerError> {
        let mut instrs: Vec<u8> = vec![]; 
        let mut instr1: u8 = 0;
        instr1 |= self.opcode_from_opid(opid)? << 2;
        instrs.push(instr1);

        instrs.push(0b0000_0000);

        return Ok(instrs);
    }

    fn assemble_double_op(&mut self, op: Stmt) -> Result<Vec<u8>, AssemblerError> {
        let mut instrs: Vec<u8> = vec![]; 
        let mut cleanup: Vec<u8> = vec![]; // for memory ops that require cleanup
        if let Stmt::DoubleOperation { opid, mode, dest, src, operand_length } = op {

            let mut operand_length = operand_length;
            if mode == DoubleMode::Rmi || mode == DoubleMode::Mir {
                operand_length = OperandLength::Unsigned16;
            }

            let mut instr1: u8 = 0;
            instr1 |= self.opcode_from_opid(opid)? << 2;
            let modebinary = self.dbmode_convert(mode.clone())?;
            let mode1: u8 = get_bits_lsb(modebinary as u16, 2, 3) as u8;
            let mode2: u8 = get_bits_lsb(modebinary as u16, 0, 1) as u8;
            instr1 |= mode1;
            let mut instr2: u8 = 0;
            instr2 |= mode2 as u8;
            instr2 <<= 6;

            let mut reg1: u8 = 0;
            let mut reg2: u8 = 0;
            let mut imm: Vec<u8> = vec![];

            if mode == DoubleMode::Rr {
                reg1 = self.register_from_operand(dest)?;
                reg2 = self.register_from_operand(src)?;
            }
            else if mode == DoubleMode::Rm {
                reg1 = self.register_from_operand(dest)?;
                reg2 = self.register_from_operand(src)?;
                
                // let (register2, mut setup_instrs, mut cleanup_instrs) = self.mem_to_reg_from_operand(src)?;
                // reg2 = register2;
                // instrs.append(&mut setup_instrs);
                // cleanup.append(&mut cleanup_instrs);
            } // FINISH
            else if mode == DoubleMode::Mr {
                reg1 = self.register_from_operand(dest)?;
                reg2 = self.register_from_operand(src)?;
                //
                // let (register1, mut setup_instrs, mut cleanup_instrs) = self.mem_to_reg_from_operand(dest)?;
                // reg1 = register1;
                // reg2 = self.register_from_operand(src)?;
                // instrs.append(&mut setup_instrs);
                // cleanup.append(&mut cleanup_instrs);
            }
            else if mode == DoubleMode::Ri {
                reg1 = self.register_from_operand(dest)?;
                match operand_length {
                    OperandLength::Unsigned16 => {
                        let Operand::Immediate(Expr::Num(numexpr))  = src else 
                        {
                            return Err(AssemblerError {message: "Expected 16-bit immediate operand.".to_string()})
                        };
                        let (m1, m2) = self.get_addr_from_numexpr(numexpr)?;
                        imm.push(m1);
                        imm.push(m2);
                        
                    },
                    OperandLength::Unsigned8 => {
                        if let Operand::Immediate(Expr::Num(NumExpr::Raw(i)))  = src {
                            let immediate_val: u8 = i as u8;

                            imm.push(immediate_val);
                        } 
                        else if let Operand::Immediate(Expr::Num(NumExpr::Function(f))) = src {
                            let immediate_val: u8 = self.eval_func(*f)? as u8;
                            imm.push(immediate_val);
                        }
                        else if let Operand::Immediate(Expr::Num(NumExpr::Reference(r))) = src {
                            if self.labels.contains_key(&r) {
                                return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                            }
                            else {
                                // in consts which takes u8s
                                let immediate_val: u8 = self.get_const(&r)?;
                                imm.push(immediate_val);
                            }
                        }
                        else {
                            return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                        };
                        
                    },
                    OperandLength::Any => (),
                    OperandLength::Zero => return Err(AssemblerError { message: "Received OperandLength of zero during immediate value request".to_string() }),
                    
                }
            }
            else if mode == DoubleMode::Rmi {
                reg1 = self.register_from_operand(dest)?;
                match operand_length {
                    OperandLength::Unsigned16 => {
                        let Operand::Immediate(Expr::Num(numexpr))  = src else 
                        {
                            return Err(AssemblerError {message: "Expected 16-bit immediate operand.".to_string()})
                        };
                        let (m1, m2) = self.get_addr_from_numexpr(numexpr)?;
                        imm.push(m1);
                        imm.push(m2);
                        
                    },
                    OperandLength::Unsigned8 => {
                        if let Operand::Immediate(Expr::Num(NumExpr::Raw(i)))  = src {
                            let immediate_val: u8 = i as u8;

                            imm.push(immediate_val);
                        } 
                        else if let Operand::Immediate(Expr::Num(NumExpr::Function(f))) = src {
                            let immediate_val: u8 = self.eval_func(*f)? as u8;
                            imm.push(immediate_val);
                        }
                        else if let Operand::Immediate(Expr::Num(NumExpr::Reference(r))) = src {
                            if self.labels.contains_key(&r) {
                                return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                            }
                            else {
                                // in consts which takes u8s
                                let immediate_val: u8 = self.get_const(&r)?;
                                imm.push(immediate_val);
                            }
                        }
                        else {
                            return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                        };
                        
                    },
                    OperandLength::Any => (),
                    OperandLength::Zero => return Err(AssemblerError { message: "Received OperandLength of zero during immediate value request".to_string() }),
                    
                }
            }
            else if mode == DoubleMode::Mir {
                reg2 = self.register_from_operand(src)?;
                match operand_length {
                    OperandLength::Unsigned16 => {
                        let Operand::Immediate(Expr::Num(numexpr))  = dest else 
                        {
                            return Err(AssemblerError {message: "Expected 16-bit immediate operand.".to_string()})
                        };
                        let (m1, m2) = self.get_addr_from_numexpr(numexpr)?;
                        imm.push(m1);
                        imm.push(m2);
                        
                    },
                    OperandLength::Unsigned8 => {
                        if let Operand::Immediate(Expr::Num(NumExpr::Raw(i)))  = dest {
                            let immediate_val: u8 = i as u8;

                            imm.push(immediate_val);
                        } 
                        else if let Operand::Immediate(Expr::Num(NumExpr::Function(f))) = dest {
                            let immediate_val: u8 = self.eval_func(*f)? as u8;
                            imm.push(immediate_val);
                        }
                        else if let Operand::Immediate(Expr::Num(NumExpr::Reference(r))) = dest {
                            if self.labels.contains_key(&r) {
                                return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                            }
                            else {
                                // in consts which takes u8s
                                let immediate_val: u8 = self.get_const(&r)?;
                                imm.push(immediate_val);
                            }
                        }
                        else {
                            return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                        };
                        
                    },
                    OperandLength::Any => (),
                    OperandLength::Zero => return Err(AssemblerError { message: "Received OperandLength of zero during immediate value request".to_string() }),
                    
                }
            }

            instr2 |= reg1 << 3;
            instr2 |= reg2;




            instrs.push(instr1);
            instrs.push(instr2);
            instrs.append(&mut imm);
            instrs.append(&mut cleanup); // add cleanup instructions at end
        }
        else {
            return Err(AssemblerError { message: "Unable to parse DoubleOperation".to_string() });
        }

        return Ok(instrs);
    }

    fn assemble_single_op(&mut self, op: Stmt) -> Result<Vec<u8>, AssemblerError> {
        let mut instrs: Vec<u8> = vec![];
        // let mut cleanup: Vec<u8> = vec![]; // for memory ops that require cleanup
        println!("Single op: {:?}", op);

        if let Stmt::SingleOperation { opid, mode, operand, operand_length: operand_length1 } = op {


            let mut operand_length = operand_length1;
            if mode == SingleMode::Mi && operand_length == OperandLength::Unsigned8 {
                operand_length = OperandLength::Unsigned16;
            }

            let mut instr1: u8 = 0;
            instr1 |= self.opcode_from_opid(opid)? << 2;
            let modebinary = self.smode_convert(mode.clone())?;
            println!("Modebinary: {:08b}", modebinary);
            let mode1: u8 = get_bits_lsb(modebinary as u16, 2, 3) as u8;
            let mode2: u8 = get_bits_lsb(modebinary as u16, 0, 1) as u8;
            instr1 |= mode1;
            let mut instr2: u8 = 0;
            instr2 |= mode2 as u8;
            instr2 <<= 6;

            let mut reg1: u8 = 0;
            let mut reg2: u8 = 0;

            let mut imm: Vec<u8> = vec![];

            if mode == SingleMode::R {
                reg1 = self.register_from_operand(operand)?;
            }
            else if mode == SingleMode::M {
                // let (register1, mut setup_instrs, mut cleanup_instrs) = self.mem_to_reg_from_operand(operand)?;
                // reg1 = register1;

                
                
                // instrs.append(&mut setup_instrs);
                // cleanup.append(&mut cleanup_instrs);

                match operand {
                    Operand::Immediate(o) => match operand_length {
                        OperandLength::Unsigned16 => {
                            let Expr::Num(numexpr)  = o else 
                            {
                                return Err(AssemblerError {message: "Expected 16-bit immediate operand.".to_string()})
                            };
                            let (m1, m2) = self.get_addr_from_numexpr(numexpr)?;
                            imm.push(m1);
                            imm.push(m2);
                            
                        },
                        OperandLength::Unsigned8 => {
                            if let Expr::Num(NumExpr::Raw(i))  = o {
                                let immediate_val: u8 = i as u8;

                                imm.push(immediate_val);
                            } 
                            else if let Expr::Num(NumExpr::Function(f)) = o {
                                let immediate_val: u8 = self.eval_func(*f)? as u8;
                                imm.push(immediate_val);
                            }
                            else {
                                return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                            };
                            
                        },
                        OperandLength::Any => (),
                        OperandLength::Zero => return Err(AssemblerError { message: "Received OperandLength of zero during immediate value request".to_string() }),
                        
                    },
                    Operand::Register(r) => {
                        reg1 = r;
                    }
                }
            }
            else if mode == SingleMode::I {
                match operand_length {
                    OperandLength::Unsigned16 => {
                        let Operand::Immediate(Expr::Num(numexpr))  = operand else 
                        {
                            return Err(AssemblerError {message: "Expected 16-bit immediate operand.".to_string()})
                        };
                        let (m1, m2) = self.get_addr_from_numexpr(numexpr)?;
                        imm.push(m1);
                        imm.push(m2);
                        
                    },
                    OperandLength::Unsigned8 => {
                        if let Operand::Immediate(Expr::Num(NumExpr::Raw(i)))  = operand {
                            let immediate_val: u8 = i as u8;

                            imm.push(immediate_val);
                        } 
                        else if let Operand::Immediate(Expr::Num(NumExpr::Function(f))) = operand {
                            let immediate_val: u8 = self.eval_func(*f)? as u8;
                            imm.push(immediate_val);
                        }
                        else {
                            return Err(AssemblerError {message: "Expected 8-bit immediate operand.".to_string()})
                        };
                        
                    },
                    OperandLength::Any => (),
                    OperandLength::Zero => return Err(AssemblerError { message: "Received OperandLength of zero during immediate value request".to_string() }),
                    
                }
            }
            else if mode == SingleMode::Mi {
                let Operand::Immediate(Expr::Num(numexpr))  = operand else 
                {
                    return Err(AssemblerError {message: "Expected 16-bit immediate operand.".to_string()})
                };
                let (m1, m2) = self.get_addr_from_numexpr(numexpr)?;
                imm.push(m1);
                imm.push(m2);
            }

            instr2 |= reg1 << 3;
            instr2 |= reg2;

            println!("instr1: {:08b}", instr1);
            println!("instr2: {:08b}", instr2);
            println!("imm: {:?}", imm);




            instrs.push(instr1);
            instrs.push(instr2);
            if !imm.is_empty() {
                instrs.append(&mut imm);
            }
            // instrs.append(&mut cleanup);
        }
        else {
            return Err(AssemblerError { message: "Unable to parse SingleOperation".to_string() });
        }

        return Ok(instrs);
    }

    fn get_label(&mut self, label: &str) -> Result<u16, AssemblerError> {
        return Ok(match self.labels.get(label) {
            Some(u) => *u,
            None => match self.mode {
                AssembleMode::Assemble => return Err(AssemblerError { message: "Attempted to get nonexistent label".to_string() }),
                AssembleMode::CountBytes => 0,
            }
        });
    }

    fn get_const(&mut self, label: &str) -> Result<u8, AssemblerError> {
        return Ok(match self.consts.get(label) {
            Some(u) => *u,
            None => match self.mode {
                AssembleMode::Assemble => return Err(AssemblerError { message: "Attempted to get nonexistent const".to_string() }),
                AssembleMode::CountBytes => 0,
            }
        });
    }

    fn label(&mut self, label: String) -> Result<Vec<u8>, AssemblerError> { // will return a blank u8 vec
        if self.consts.contains_key(&label) {
            return Err(AssemblerError { message: "Attempt to create label w/ constant".to_string() })
        }
        self.labels.insert(label, self.pc);
        return Ok(vec![]);
    }

    fn eval_func(&mut self, func: Function) -> Result<i64, AssemblerError> {
        match func {
            Function::Hi(numexp) => {
                let addr = self.eval_expr(Expr::Num(numexp))? as u16;
                return Ok(get_bits_msb(addr, 0, 7) as i64);

            },
            Function::Lo(numexp) => {
                let addr = self.eval_expr(Expr::Num(numexp))? as u16;
                return Ok(get_bits_lsb(addr, 0, 7) as i64);

            }
        }
    }

    fn eval_num_bin_op(&mut self, a: NumExpr, op: BinaryOp, b: NumExpr) -> Result<i64, AssemblerError> {
        match op {
            BinaryOp::Add => {
                return Ok(self.eval_expr(Expr::Num(a))? + self.eval_expr(Expr::Num(b))?);
            },
            BinaryOp::Div => {
                return Ok(self.eval_expr(Expr::Num(a))? / self.eval_expr(Expr::Num(b))?);
            },
            BinaryOp::Mul => {
                return Ok(self.eval_expr(Expr::Num(a))? * self.eval_expr(Expr::Num(b))?);
            },
            BinaryOp::Sub => {
                return Ok(self.eval_expr(Expr::Num(a))? - self.eval_expr(Expr::Num(b))?);
            }
        }
    }

    fn eval_expr(&mut self, expr: Expr) -> Result<i64, AssemblerError> {
        return Ok(match expr {
            Expr::Str(_) => return Err(AssemblerError { message: "Couldn't turn string into one i64.".to_string() }),
            Expr::Num(n) => match n {
                NumExpr::Raw(i) => i,
                NumExpr::Reference(r) => self.get_label(r.as_str())? as i64,
                NumExpr::Function(f) => self.eval_func(*f)?,
                NumExpr::BinaryOperation { a, operand, b } => self.eval_num_bin_op(*a, operand, *b)?,
            },
            _ => return Err(AssemblerError { message: "Couldn't understand expr".to_string() })

        });
    }

    fn parse_signal(&mut self, name: String, args: Vec<Expr>) -> Result<Vec<u8>, AssemblerError> {
        let mut returner: Vec<u8> = vec![];
        if name == "org" {
            if args.is_empty() || args.len() > 1 {
                return Err(AssemblerError { message: "org signal arg count incorrect".to_string() });
            }
            else {
                let new_pos = match &args[0] {
                    Expr::Num(n) => match n {
                        NumExpr::Reference(s) => {
                            self.get_label(s)?
                        },
                        NumExpr::Raw(i) => *i as u16,
                        NumExpr::Function(f) => self.eval_func(*f.clone())? as u16,
                        NumExpr::BinaryOperation { a, operand, b } => self.eval_num_bin_op(*a.clone(), operand.clone(), *b.clone())? as u16,

                    },
                    _ => return Err(AssemblerError { message: "org signal received incorrect arg".to_string() })
                };
                self.pc = new_pos;
                self.current_pos = new_pos;
            }
        }
        else if name == "byte" {
            if args.is_empty() {
                return Err(AssemblerError { message: "byte signal arg count incorrect".to_string() });
            }
            else {
                for arg in args {
                    let byte = self.eval_expr(arg)?;
                    if 0 > byte || 255 < byte {
                        return Err(AssemblerError { message: format!(".byte only takes 1-byte args, received {:?}", byte) })
                    }
                    else {
                        returner.push(byte as u8);
                    }
                }
            }
        }
        else if name == "const" {
            if args.len() != 2 {
                return Err(AssemblerError { message: "const signal arg count incorrect".to_string() });
            }
            else {
                    
                let cons = (&args[0]).clone();
                let val = (&args[1]).clone();
                

                if let Expr::Str(StrExpr::Raw(s)) = cons {
                    if self.labels.contains_key(&s) {
                        return Err(AssemblerError { message: "Attempt to create constant w/ label".to_string() })
                    }
                    let byte = self.eval_expr(val)?;
                    if 0 > byte || 255 < byte {
                        return Err(AssemblerError { message: format!(".const only takes 1-byte args, received {:?}", byte) })
                    }
                    else {
                        self.consts.insert(s, byte as u8);
                    }
                }
                else {
                    return Err(AssemblerError { message: format!("const signal takes arg1 = string, got {:?}", args[0]) })
                }
            }
        }
        return Ok(returner);
    }


    fn walk(&mut self) -> Result<Option<HashMap<u16, Vec<u8>>>, AssemblerError> {


        let mut byte_segments: HashMap<u16, Vec<u8>> = HashMap::new();
        self.current_pos = self.pc;
        
        let program = self.program.clone();



        for stmt in program {
            println!("Stmt: {:?}", stmt);
            let next_instructions: Vec<u8> = match stmt {
                Stmt::DoubleOperation { opid, mode, dest, src, operand_length } => self.assemble_double_op(Stmt::DoubleOperation { opid, mode, dest, src, operand_length })?,
                Stmt::SingleOperation { opid, mode, operand, operand_length } => self.assemble_single_op(Stmt::SingleOperation { opid, mode, operand, operand_length })?,
                Stmt::ZeroOperation { opid } => self.assemble_zero_op(opid)?,
                Stmt::Label(s) => self.label(s)?,
                Stmt::End => vec![],
                Stmt::Comment(_) => vec![],
                Stmt::Signal { name, args } => self.parse_signal(name, args)?,
                Stmt::Newline => vec![],
                _ => return Err(AssemblerError { message: "Unexpected stmt".to_string() }),
            };
            if !next_instructions.is_empty() {
                println!("Next instructions:");
                for inst in next_instructions.clone() {
                    print!("(0x{:x}): 0b{:08b},\n", self.current_pos, inst);
                }
                print!("\n");
                self.inc_pc(next_instructions.len() as u16);
                if self.mode == AssembleMode::Assemble {
                    byte_segments.entry(self.current_pos).or_insert_with(||Vec::new()).extend(next_instructions);
                }
                
            }
            else {
                println!("Empty instructions");
            }
        }

        print!("\nLabels:");
        for (labels, labelu) in self.labels.clone() {
            print!("\n[\"{}\": {:0x}]", labels, labelu);
        }
        print!("\n");
        print!("\nConsts:");
        for (labels, labelu) in self.consts.clone() {
            print!("\n[\"{}\": {:08b}]", labels, labelu);
        }
        print!("\n");
        

        return Ok(match self.mode {
            AssembleMode::CountBytes => None,
            AssembleMode::Assemble => Some(byte_segments),
        });
    }

    pub fn assemble(&mut self) -> Result<HashMap<u16, Vec<u8>>, AssemblerError> {
        // save current state
        let orig_pc = self.pc;
        let orig_pos = self.current_pos;


        // first pass
        self.mode = AssembleMode::CountBytes;
        self.walk()?;
        let lbls = self.labels.clone();


        // second pass
        self.mode = AssembleMode::Assemble;

        self.pc = orig_pc;
        self.current_pos = orig_pos;
        self.labels = lbls;

        return Ok(match self.walk()? {
            Some(s) => s,
            None => return Err(AssemblerError { message: "Assembling returned none".to_string() })
        });
    }
}

