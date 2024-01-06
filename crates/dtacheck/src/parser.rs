use std::ops::Range;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use derive_more::IsVariant;
use derive_more::Unwrap;

use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::linter::Lint;

#[derive(Default)]
struct Parser<'a> {
    cursor: usize,
    brace_stack: Vec<Token>,
    tokens: &'a [Token],
    diagnostics: Vec<ParseLint>,
}

#[derive(Debug)]
pub enum ParseLint {
    UnmatchedBrace(Range<usize>, Range<usize>),
    GenericError(Range<usize>),
}

impl Lint for ParseLint {
    fn to_codespan(&self, id: usize) -> Diagnostic<usize> {
        match self {
            Self::UnmatchedBrace(opening, closing) => Diagnostic::error()
                .with_message("unmatched delimiter")
                .with_labels(vec![
                    Label::primary(id, closing.clone()).with_message("unexpected token"),
                    Label::primary(id, opening.clone()).with_message("unmatched delimiter"),
                ]),
            Self::GenericError(span) => Diagnostic::error()
                .with_message("unexpected token")
                .with_labels(vec![
                    Label::primary(id, span.clone()).with_message("unexpected token")
                ]),
        }
    }
}

type ParseResult<T> = Result<T, ParseLint>;

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }

    fn bump(&mut self, amount: usize) {
        self.cursor += amount;
    }

    fn lookahead(&self, amount: usize) -> Token {
        self.tokens[self.cursor + amount].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.cursor - 1].clone()
    }

    fn eat(&mut self, f: fn(&TokenKind) -> bool) -> ParseResult<Token> {
        let token = self.lookahead(0);

        if f(&token.kind) {
            self.bump(1);
            Ok(token)
        } else {
            Err(ParseLint::GenericError(token.span))
        }
    }

    fn eat_open_brace(&mut self, f: fn(&TokenKind) -> bool) -> ParseResult<Token> {
        let token = self.lookahead(0);

        if f(&token.kind) {
            self.brace_stack.push(token.clone());
            self.bump(1);
            Ok(token)
        } else {
            Err(ParseLint::GenericError(token.span))
        }
    }

    fn eat_if(&mut self, f: fn(&TokenKind) -> bool) -> bool {
        let token = self.lookahead(0);

        if f(&token.kind) {
            self.bump(1);
            true
        } else {
            false
        }
    }

    fn eat_if_open_brace(&mut self, f: fn(&TokenKind) -> bool) -> bool {
        let token = self.lookahead(0);

        if f(&token.kind) {
            self.brace_stack.push(token.clone());
            self.bump(1);
            true
        } else {
            false
        }
    }

    // i seriously can not think of a better way to write this
    //
    // PRs welcome
    #[allow(clippy::if_same_then_else)]
    fn parse_node(&mut self) -> ParseResult<Node> {
        if self.eat_if(TokenKind::is_int) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if(TokenKind::is_float) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if(TokenKind::is_var) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if(TokenKind::is_sym) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if(TokenKind::is_unhandled) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if(TokenKind::is_if_def) {
            let span = self.previous().span;
            let sym = self.eat(TokenKind::is_sym)?;
            Ok(Node::new_ifdef(span, sym))
        } else if self.eat_if(TokenKind::is_else) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if(TokenKind::is_end_if) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if_open_brace(TokenKind::is_l_paren) {
            let lower_span = self.previous().span;
            let array = self.parse_list(TokenKind::is_r_paren)?;
            let upper_span = self.previous().span;
            Ok(Node::new_array(array, lower_span.start..upper_span.end))
        } else if self.eat_if_open_brace(TokenKind::is_l_bracket) {
            let lower_span = self.previous().span;
            let array = self.parse_list(TokenKind::is_r_bracket)?;
            let upper_span = self.previous().span;
            Ok(Node::new_prop(array, lower_span.start..upper_span.end))
        } else if self.eat_if(TokenKind::is_string) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if_open_brace(TokenKind::is_l_brace) {
            let lower_span = self.previous().span;
            let array = self.parse_list(TokenKind::is_r_brace)?;
            let upper_span = self.previous().span;
            Ok(Node::new_stmt(array, lower_span.start..upper_span.end))
        } else if self.eat_if(TokenKind::is_define) {
            let span = self.previous().span;
            let sym = self.eat(TokenKind::is_sym)?;
            self.eat_open_brace(TokenKind::is_l_paren)?;
            let array = self.parse_list(TokenKind::is_r_paren)?;
            Ok(Node::new_define(span, sym, array))
        } else if self.eat_if(TokenKind::is_include) {
            let span = self.previous().span;
            let sym = self.eat(TokenKind::is_sym)?;
            Ok(Node::new_include(span, sym))
        } else if self.eat_if(TokenKind::is_merge) {
            let span = self.previous().span;
            let sym = self.eat(TokenKind::is_sym)?;
            Ok(Node::new_merge(span, sym))
        } else if self.eat_if(TokenKind::is_if_n_def) {
            let span = self.previous().span;
            let sym = self.eat(TokenKind::is_sym)?;
            Ok(Node::new_ifndef(span, sym))
        } else if self.eat_if(TokenKind::is_autorun) {
            Ok(Node::from(self.previous()))
        } else if self.eat_if(TokenKind::is_un_def) {
            let span = self.previous().span;
            let sym = self.eat(TokenKind::is_sym)?;
            Ok(Node::new_undef(span, sym))
        } else {
            Err(ParseLint::GenericError(self.lookahead(0).span))
        }
    }

    fn parse_list(&mut self, stop: fn(&TokenKind) -> bool) -> ParseResult<Vec<Node>> {
        let mut nodes = Vec::new();
        loop {
            if self.eat_if(stop) {
                if self.previous().kind != TokenKind::Eof {
                    self.brace_stack.pop().unwrap();
                }
                break;
            }
            match self.parse_node() {
                Ok(x) => nodes.push(x),
                Err(e) => {
                    if !self.brace_stack.is_empty() {
                        let token = self.lookahead(0);
                        let unmatched = self.brace_stack.last().unwrap().span.clone();
                        let current = token.span.clone();
                        let diag = ParseLint::UnmatchedBrace(unmatched, current);

                        if token.kind.is_r_bracket()
                            || token.kind.is_r_paren()
                            || token.kind.is_r_brace()
                        {
                            self.diagnostics.push(diag);
                            self.bump(1);
                            self.brace_stack.pop().unwrap();
                            break;
                        }

                        if token.kind.is_eof() {
                            self.diagnostics.push(diag);
                            self.brace_stack.pop().unwrap();
                            break;
                        }
                    }

                    return Err(e);
                }
            }
        }
        Ok(nodes)
    }
}

pub fn parse(tokens: &[Token]) -> (Result<Vec<Node>, ()>, Vec<ParseLint>) {
    let mut parser = Parser::new(tokens);
    let parse_result = parser.parse_list(TokenKind::is_eof);
    let mut diagnostics = parser.diagnostics;

    let res = match parse_result {
        Ok(r) => Ok(r),
        Err(e) => {
            diagnostics.push(e);
            Err(())
        }
    };

    (res, diagnostics)
}

#[derive(Debug, IsVariant, Unwrap)]
pub enum NodeKind {
    Int(i32),
    Float(f32),
    Var(String),
    Symbol(String),
    Unhandled,
    IfDef(String),
    Else,
    EndIf,
    Array(Vec<Node>),
    Stmt(Vec<Node>),
    String(String),
    Prop(Vec<Node>),
    Define(String, Vec<Node>),
    Include(String),
    Merge(String),
    IfNDef(String),
    Autorun,
    Undef(String),
}

#[derive(Debug)]
#[allow(unused)]
pub struct Node {
    pub kind: NodeKind,
    pub span: Range<usize>,
}

#[allow(clippy::reversed_empty_ranges)]
fn combine_node_spans(list: &[Node]) -> Range<usize> {
    list.iter()
        .map(|x| &x.span)
        .fold(usize::MAX..0, |current, next| {
            let start = current.start.min(next.start);
            let end = current.end.max(next.end);
            start..end
        })
}

impl Node {
    fn new_array(list: Vec<Node>, span: Range<usize>) -> Node {
        Node {
            kind: NodeKind::Array(list),
            span,
        }
    }

    fn new_stmt(list: Vec<Node>, span: Range<usize>) -> Node {
        Node {
            kind: NodeKind::Stmt(list),
            span,
        }
    }

    fn new_prop(list: Vec<Node>, span: Range<usize>) -> Node {
        Node {
            kind: NodeKind::Prop(list),
            span,
        }
    }

    fn new_define(span: Range<usize>, sym: Token, array: Vec<Node>) -> Node {
        let vec_span = combine_node_spans(&array);
        let span = span.start..vec_span.end.min(sym.span.end);
        Node {
            kind: NodeKind::Define(sym.kind.unwrap_sym(), array),
            span,
        }
    }

    fn new_ifdef(span: Range<usize>, sym: Token) -> Node {
        let span = span.start..sym.span.end;
        Node {
            kind: NodeKind::IfDef(sym.kind.unwrap_sym()),
            span,
        }
    }
    fn new_ifndef(span: Range<usize>, sym: Token) -> Node {
        let span = span.start..sym.span.end;
        Node {
            kind: NodeKind::IfNDef(sym.kind.unwrap_sym()),
            span,
        }
    }
    fn new_include(span: Range<usize>, sym: Token) -> Node {
        let span = span.start..sym.span.end;
        Node {
            kind: NodeKind::Include(sym.kind.unwrap_sym()),
            span,
        }
    }
    fn new_merge(span: Range<usize>, sym: Token) -> Node {
        let span = span.start..sym.span.end;
        Node {
            kind: NodeKind::Merge(sym.kind.unwrap_sym()),
            span,
        }
    }
    fn new_undef(span: Range<usize>, sym: Token) -> Node {
        let span = span.start..sym.span.end;
        Node {
            kind: NodeKind::Undef(sym.kind.unwrap_sym()),
            span,
        }
    }

    pub fn is_preproc(&self) -> bool {
        self.kind.is_if_def()
            || self.kind.is_else()
            || self.kind.is_end_if()
            || self.kind.is_define()
            || self.kind.is_include()
            || self.kind.is_merge()
            || self.kind.is_if_n_def()
            || self.kind.is_autorun()
            || self.kind.is_undef()
    }
}

impl From<Token> for Node {
    fn from(value: Token) -> Self {
        match value.kind {
            TokenKind::Sym(s) => Node {
                kind: NodeKind::Symbol(s),
                span: value.span,
            },
            TokenKind::Int(s) => Node {
                kind: NodeKind::Int(s),
                span: value.span,
            },
            TokenKind::Float(s) => Node {
                kind: NodeKind::Float(s),
                span: value.span,
            },
            TokenKind::String(s) => Node {
                kind: NodeKind::String(s),
                span: value.span,
            },
            TokenKind::Var(s) => Node {
                kind: NodeKind::Var(s),
                span: value.span,
            },
            TokenKind::Else => Node {
                kind: NodeKind::Else,
                span: value.span,
            },
            TokenKind::EndIf => Node {
                kind: NodeKind::EndIf,
                span: value.span,
            },
            TokenKind::Unhandled => Node {
                kind: NodeKind::Unhandled,
                span: value.span,
            },
            TokenKind::Autorun => Node {
                kind: NodeKind::Autorun,
                span: value.span,
            },
            _ => unreachable!(),
        }
    }
}
