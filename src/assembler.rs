

/*
CORE IDEA:

[OPERATION] [MODE] [REGS] [OPTIONAL IMMEDIATE]
i.e.
move rr r0 r1; -> moves r1 to r0


FUNCTIONS:
fnc [func name]:

(function stuff)

end



*/


#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Int(i32),
    Str(String),
    Comment(String),
    SEMICOLON,
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