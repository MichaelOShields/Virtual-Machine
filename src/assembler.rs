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

    let mut assembler = Assembler::new(parser);
    assembler.assemble();

    // let mut first = match parser.next_stmt() {
    //         Ok(s) => s,
    //         Err(e) => panic!("Got PE {:?}", e),
    //     };
    // println!("First: {:?}", first);

    // while first != Stmt::End {
    //     if first != Stmt::Newline {
    //         println!("Stmt: {:?}", first);  
    //     };
    //     first = match parser.next_stmt() {
    //         Ok(s) => s,
    //         Err(e) => {println!("Error: {:?}", e); break;}
    //     };
    // }

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
pub enum OpKind {
    Double,
    Single,
    Zero,
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
    ops: HashMap<String, OpKind>,
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

        let mut ops: HashMap<String, OpKind> = HashMap::new();
        ops.insert("nop".to_string(), OpKind::Zero);
        ops.insert("ret".to_string(), OpKind::Zero);
        ops.insert("hlt".to_string(), OpKind::Zero);

        ops.insert("not".to_string(), OpKind::Single);

        ops.insert("jmp".to_string(), OpKind::Single);
        ops.insert("jz".to_string(), OpKind::Single);
        ops.insert("jc".to_string(), OpKind::Single);
        ops.insert("jo".to_string(), OpKind::Single);
        ops.insert("js".to_string(), OpKind::Single);
        ops.insert("jnz".to_string(), OpKind::Single);
        ops.insert("jg".to_string(), OpKind::Single);
        ops.insert("jl".to_string(), OpKind::Single);

        ops.insert("push".to_string(), OpKind::Single);
        ops.insert("pop".to_string(), OpKind::Single);
        ops.insert("call".to_string(), OpKind::Single);

        ops.insert("shl".to_string(), OpKind::Single);
        ops.insert("shr".to_string(), OpKind::Single);
        ops.insert("sar".to_string(), OpKind::Single);

        ops.insert("ssp".to_string(), OpKind::Single);
        ops.insert("skip".to_string(), OpKind::Single);


        ops.insert("mov".to_string(), OpKind::Double);

        ops.insert("add".to_string(), OpKind::Double);
        ops.insert("sub".to_string(), OpKind::Double);
        ops.insert("mul".to_string(), OpKind::Double);
        ops.insert("div".to_string(), OpKind::Double);
        ops.insert("mod".to_string(), OpKind::Double);

        ops.insert("and".to_string(), OpKind::Double);
        ops.insert("or".to_string(), OpKind::Double);
        ops.insert("xor".to_string(), OpKind::Double);

        ops.insert("cmp".to_string(), OpKind::Double);



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
        self.expect_ident()?; // opid
        let mode_ident = self.expect_ident()?;
        // println!("mode: {:?}", mode_ident);
        let mode = match self.double_modes.get(&mode_ident) {
            Some(m) => m,
            None => return Err(ParserError { message: "Unable to get mode".to_string() }),
        }.clone();
        let dest = self.expect_operand()
            .map_err(|_| ParserError { message: "Expected dest operand".into() })?;
        self.basic_token(Token::COMMA)?;
        let src = self.expect_operand()
            .map_err(|_| ParserError { message: "Expected src operand".into() })?;
        return Ok(Stmt::DoubleOperation { opid, mode, dest, src })
    }

    fn parse_single_op(&mut self, opid: String) -> Result<Stmt, ParserError> {
        self.expect_ident()?; // opid
        let mode_ident = self.expect_ident()?;
        // println!("mode: {:?}", mode_ident);
        let mode = match self.single_modes.get(match (&mode_ident).as_str() {
            "r" => &'r',
            "i" => &'i',
            "m" => &'m',
            _ => return Err(ParserError { message: format!("Received unexpected mode {:?}", mode_ident) }),
        }) {
            Some(m) => m,
            None => return Err(ParserError { message: "Unable to get mode".to_string() }),
        }.clone();
        let operand = self.expect_operand()
            .map_err(|_| ParserError { message: "Expected single operand".into() })?;
        return Ok(Stmt::SingleOperation { opid, mode, operand })
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
        let mut taking_args = self.taking_next_args()?;

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
                _ => return Err(ParserError { message: format!("Expected NumExpr, got {:?}", self.current_token) }),
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


    fn parse_signal(&mut self) -> Result<Stmt, ParserError> {
        self.basic_token(Token::PERIOD)?;
        let sigid = match &self.current_token {
            Token::Ident(s) => s.clone(),
            _ => return Err(ParserError { message: format!("Expected signal identifier, got {:?}", self.current_token) })
        };
        self.advance()?;
        return Ok(self.parse_sig(sigid.to_string())?);
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
            match self.ops.get(s.as_str()) {
                Some(OpKind::Double) => return Ok(self.parse_double_op(s.clone())?),
                Some(OpKind::Single) => return Ok(self.parse_single_op(s.clone())?),
                Some(OpKind::Zero) => return Ok(self.parse_zero_op(s.clone())?),
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
}


// assembler will reserve R6 and R7 for under the hood memory ops

#[derive(Debug)]
pub struct AssemblerError {
    message: String,
}

#[derive(Debug, PartialEq)]
pub struct Assembler {
    pc: u16,
    output: Vec<u8>,
    labels: HashMap<String, u16>,
    parser: Parser,
}

impl Assembler {
    pub fn new(parser: Parser) -> Self {
        Self {
            pc: 0_u16,
            output: vec![],
            labels: HashMap::new(),
            parser,
        }
    }

    fn get_stmt(&mut self) -> Result<Stmt, AssemblerError> {
        return Ok(match self.parser.next_stmt() {
            Ok(s) => s,
            Err(e) => return Err(AssemblerError { message: format!("Got ParserError {:?}", e) }),

        });
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
            _ => return Err(AssemblerError { message: "Unable to parse mode".to_string() }),
        });
    }

    fn register_from_operand(&mut self, dest: Operand) -> Result<u8, AssemblerError> {
        return Ok(match dest {
            Operand::Register(num) => num,
            _ => return Err(AssemblerError { message: "Expected register".to_string() })
        });
    }

    fn mem_from_operand(&mut self, mem: Operand) -> Result<(u8, Vec<u8>), AssemblerError> { // result is register to grab mem addr, vec<u8> is just loading the mem addr into R6 and R7
        let reg: u8 = 6;
        let mut instrs: Vec<u8> = vec![];
        match mem {
            Operand::Immediate(e) => match e {
                Expr::Num(numex) => match numex {
                    NumExpr::Reference(r) => {
                        if !self.labels.contains_key(&r) {
                            return Err(AssemblerError { message: "Attempted to reference a nonexistent label".to_string() })
                        };
                        let mem_addr: u16 = *self.labels.get(&r).unwrap();
                        let m1: u8 = get_bits_msb(mem_addr, 0, 7) as u8;
                        let m2: u8 = get_bits_lsb(mem_addr, 0, 7) as u8;
                        let movem1toreg6stmt = Stmt::DoubleOperation { opid: "mov".to_string(), mode: DoubleMode::Ri, dest: Operand::Register(6), src: Operand::Immediate(Expr::Num(NumExpr::Raw(m1 as i64))) };
                        let movem2toreg7stmt = Stmt::DoubleOperation { opid: "mov".to_string(), mode: DoubleMode::Ri, dest: Operand::Register(7), src: Operand::Immediate(Expr::Num(NumExpr::Raw(m2 as i64))) };
                        let mut loadingm1 = self.assemble_double_op(movem1toreg6stmt)?;
                        let mut loadingm2 = self.assemble_double_op(movem2toreg7stmt)?;
                        instrs.append(&mut loadingm1);
                        instrs.append(&mut loadingm2);

                        return Ok((reg, instrs));



                    },
                    _ => return Err(AssemblerError { message: "Got unexpected raw number when parsing memory operand".to_string() })
                },
                _ => return Err(AssemblerError { message: "Received unexpected immediate operand expression type.".to_string() })
            },
            Operand::Register(n) => {return Ok((n, vec![]))},
        };

    }

    fn assemble_double_op(&mut self, op: Stmt) -> Result<Vec<u8>, AssemblerError> {
        let mut instrs: Vec<u8> = vec![]; 
        if let Stmt::DoubleOperation { opid, mode, dest, src } = op {
            let mut instr1: u8 = 0;
            instr1 |= self.opcode_from_opid(opid)? << 2;
            let modebinary = self.dbmode_convert(mode.clone())?;
            let mode1: u8 = get_bits_msb(modebinary as u16, 0, 1) as u8;
            let mode2: u8 = get_bits_msb(modebinary as u16, 2, 3) as u8;
            instr1 |= mode1;
            let mut instr2: u8 = 0;
            instr2 |= mode2 as u8;
            instr2 <<= 6;

            let mut reg1: u8 = 0;
            let mut reg2: u8 = 0;

            if mode == DoubleMode::Rr {
                reg1 = self.register_from_operand(dest)?;
                reg2 = self.register_from_operand(src)?;
            }
            else if mode == DoubleMode::Rm {
                reg1 = self.register_from_operand(dest)?;
                let (register2, mut setup_instrs) = self.mem_from_operand(src)?;
                reg2 = register2;
                instrs.append(&mut setup_instrs);
            }

            instr2 |= reg1 << 3;
            instr2 |= reg2;




            instrs.push(instr1);
            instrs.push(instr2);
        }
        else {
            return Err(AssemblerError { message: "Unable to parse DoubleOperation".to_string() });
        }

        return Ok(instrs);
    }


    pub fn assemble(&mut self) -> Result<(Vec<u8>, HashMap<String, u16>), AssemblerError>{
        let mut instructions: Vec<u8> = vec![];
        let mut first: Stmt = self.get_stmt()?;
        while first != Stmt::End {
            let next_instructions: Option<Vec<u8>> = match first {
                Stmt::DoubleOperation { opid, mode, dest, src } => Some(self.assemble_double_op(Stmt::DoubleOperation { opid, mode, dest, src })?),
                _ => None,
            };
            match next_instructions {
                Some(mut i) => instructions.append(&mut i),
                None => (),
            }
            
            for inst in instructions.clone() {
                println!("instruction: {:08b}", inst);
            }
            first = self.get_stmt()?;
        }

        return Ok((instructions, self.labels.clone()));
    }
}