use std::ops::Range;

use derive_more::IsVariant;
use derive_more::Unwrap;
use logos::Lexer;
use logos::Logos;

// do not try to understand the regex here
#[derive(Logos, Debug, PartialEq, IsVariant, Unwrap, Clone)]
pub enum TokenKind {
    #[token("kDataUnhandled")]
    Unhandled,
    #[token("#ifdef")]
    IfDef,
    #[token("#else")]
    Else,
    #[token("#endif")]
    EndIf,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("#define")]
    Define,
    #[token("#include")]
    Include,
    #[token("#merge")]
    Merge,
    #[token("#ifndef")]
    IfNDef,
    #[token("#autorun")]
    Autorun,
    #[token("#undef")]
    UnDef,
    #[regex(r#"[\-\+]?[0-9]+"#, |lex| lex.slice().parse().ok(), priority=2)]
    Int(i32),
    #[regex(r#"[\-\+]?[0-9]+\.[0-9]+"#, |lex| lex.slice().parse().ok(), priority=2)]
    Float(f32),
    #[regex(r#"\$[0-9a-zA-Z_]+"#, |lex| lex.slice().parse().ok())]
    Var(String),
    #[regex(r#"[^ \t\n\r\f\(\[\{\}\]\)]+"#, |lex| lex.slice().parse().ok())]
    #[regex(r#"'(?:\.|[^'])+'"#, trim_delimiters)]
    Sym(String),
    #[regex(r#""(?:\.|[^"])+""#, trim_delimiters)]
    String(String),
    #[regex(r"(;[^\n]*|[ \t\s\f\n\r])", priority = 2, callback = logos::skip)]
    Invalid,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}

fn trim_delimiters(lexer: &mut Lexer<TokenKind>) -> Option<String> {
    let slice = lexer.slice();
    let length = slice.len();
    slice[1..length - 1].parse().ok()
}

pub fn lex(data: &str) -> Vec<Token> {
    let mut tokens: Vec<_> = TokenKind::lexer(data)
        .spanned()
        .map(|(tok, span)| match tok {
            Ok(tok) => Token { kind: tok, span },
            Err(()) => Token {
                kind: TokenKind::Invalid,
                span,
            },
        })
        .collect();

    if tokens.is_empty() {
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: 0..0,
        });
    } else {
        let last = tokens.last().unwrap().span.end;
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: last..last,
        });
    }

    tokens
}
