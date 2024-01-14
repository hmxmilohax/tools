use std::collections::HashMap;
use std::ops::Range;

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;

use crate::lexer::Token;
use crate::lexer::TokenKind;
use crate::parser::Node;
use crate::parser::NodeKind;

pub trait Lint {
    fn to_codespan(&self, id: usize) -> Diagnostic<usize>;
}

pub fn lint_file(
    ast: &[Node],
    tokens: &[Token],
    funcs: &Function,
) -> Vec<Box<dyn Lint>> {
    let mut lints = Vec::new();
    lint_node(&mut lints, ast, funcs);
    lint_preprocs(&mut lints, tokens);
    lints
}

fn lint_node(lints: &mut Vec<Box<dyn Lint>>, ast: &[Node], funcs: &Function) {
    for node in ast {
        match &node.kind {
            NodeKind::Array(array)
            | NodeKind::Prop(array)
            | NodeKind::Define(_, array) => lint_node(lints, array, funcs),
            NodeKind::Stmt(array) => {
                lint_node(lints, array, funcs);

                let has_preprocessor_directive =
                    array.iter().any(Node::is_preproc);

                if !has_preprocessor_directive {
                    lint_fn_args(lints, array, node.span.clone(), funcs);
                    lint_switch_fallthrough(lints, array, node.span.clone());
                }
            }
            _ => (),
        }
    }
}

// functions

enum FunctionArgLint {
    TooManyArgs(String, Range<usize>),
    NotEnoughArgs(String, Range<usize>),
}

impl Lint for FunctionArgLint {
    fn to_codespan(&self, id: usize) -> Diagnostic<usize> {
        match self {
            Self::TooManyArgs(name, range) => Diagnostic::error()
                .with_message(format!(
                    "calling `{name}` with too many arguments"
                ))
                .with_labels(vec![Label::primary(id, range.clone())
                    .with_message("too many arguments")]),
            Self::NotEnoughArgs(name, range) => Diagnostic::error()
                .with_message(format!(
                    "calling `{name}` with too few arguments"
                ))
                .with_labels(vec![Label::primary(id, range.clone())
                    .with_message("not enough arguments")]),
        }
    }
}

pub struct Function {
    pub min_args: usize,
    pub max_args: usize,
    pub children: HashMap<String, Function>,
}

impl Default for Function {
    fn default() -> Self {
        Self {
            min_args: 0,
            max_args: usize::MAX,
            children: HashMap::default(),
        }
    }
}

impl Function {
    pub fn lookup(&self, stmt: &[Node]) -> (&Function, usize) {
        self.lookup_inner(stmt, 0)
    }

    fn lookup_inner(&self, stmt: &[Node], depth: usize) -> (&Function, usize) {
        if self.children.is_empty() {
            return (self, depth);
        };

        let Some(node) = stmt.first() else {
            return (self, depth);
        };

        let NodeKind::Symbol(ref sym) = node.kind else {
            return (self, depth);
        };

        let Some(func) = self.children.get(sym) else {
            return (self, depth);
        };

        func.lookup_inner(&stmt[1..], depth + 1)
    }

    pub fn insert(&mut self, path: &[&str], min_args: usize, max_args: usize) {
        if path.is_empty() {
            self.min_args = min_args;
            self.max_args = max_args;
            return;
        }

        if let Some(child) = self.children.get_mut(path[0]) {
            child.insert(&path[1..], min_args, max_args);
        } else {
            let mut child = Function::default();
            child.insert(&path[1..], min_args, max_args);
            self.children.insert(path[0].to_string(), child);
        }
    }
}

fn lint_fn_args(
    lints: &mut Vec<Box<dyn Lint>>,
    stmt: &[Node],
    span: Range<usize>,
    funcs: &Function,
) {
    let (func, depth) = funcs.lookup(stmt);
    let name = generate_function_name(&stmt[..depth]);
    if stmt.len() > func.max_args + depth {
        lints.push(Box::new(FunctionArgLint::TooManyArgs(name, span)));
    } else if stmt.len() < func.min_args + depth {
        lints.push(Box::new(FunctionArgLint::NotEnoughArgs(name, span)));
    }
}

fn generate_function_name(stmt: &[Node]) -> String {
    let list: Vec<&str> = stmt
        .iter()
        .map(|x| match &x.kind {
            NodeKind::Symbol(sym) => Some(sym),
            _ => None,
        })
        .take_while(Option::is_some)
        .map(|x| x.unwrap().as_str())
        .collect();

    list.join(" ")
}

// preprocesor directives
enum PreProcLint {
    Unmatched(Range<usize>),
    Extra(Range<usize>),
}

impl Lint for PreProcLint {
    fn to_codespan(&self, id: usize) -> Diagnostic<usize> {
        match self {
            Self::Unmatched(s) => Diagnostic::error()
                .with_message("unmatched preprocessing directive")
                .with_labels(vec![Label::primary(id, s.clone())]),
            Self::Extra(s) => Diagnostic::error()
                .with_message("extraneous preprocessing directive")
                .with_labels(vec![Label::primary(id, s.clone())]),
        }
    }
}

fn lint_preprocs(lints: &mut Vec<Box<dyn Lint>>, tokens: &[Token]) {
    let mut directive_stack: Vec<(Range<usize>, bool)> = Vec::new();
    for token in tokens {
        match token.kind {
            TokenKind::IfNDef | TokenKind::IfDef => {
                directive_stack.push((token.span.clone(), false));
            }
            TokenKind::Else => {
                if let Some(entry) = directive_stack.pop() {
                    if entry.1 {
                        lints.push(Box::new(PreProcLint::Extra(
                            token.span.clone(),
                        )));
                    }
                    directive_stack.push((token.span.clone(), true));
                } else {
                    lints
                        .push(Box::new(PreProcLint::Extra(token.span.clone())));
                }
            }
            TokenKind::EndIf => {
                if directive_stack.pop().is_none() {
                    lints
                        .push(Box::new(PreProcLint::Extra(token.span.clone())));
                }
            }
            _ => (),
        }
    }

    for lint in directive_stack {
        lints.push(Box::new(PreProcLint::Unmatched(lint.0)));
    }
}

// switch fallthough

struct SwitchFallthroughLint(Range<usize>, Range<usize>);

impl Lint for SwitchFallthroughLint {
    fn to_codespan(&self, id: usize) -> Diagnostic<usize> {
        Diagnostic::warning()
            .with_message("missing fallthrough for switch")
            .with_labels(vec![
                Label::primary(id, self.0.clone()),
                Label::secondary(id, self.1.clone())
                    .with_message("consider adding a fallthrough node here"),
            ])
    }
}

fn lint_switch_fallthrough(
    lints: &mut Vec<Box<dyn Lint>>,
    stmt: &[Node],
    span: Range<usize>,
) {
    if stmt.is_empty() {
        return;
    }

    let NodeKind::Symbol(ref sym) = stmt[0].kind else {
        return;
    };

    if sym != "switch" {
        return;
    }

    let Some(last_node) = stmt.last() else {
        return;
    };

    if last_node.kind.is_array() {
        let pos = span.end - 1;
        lints.push(Box::new(SwitchFallthroughLint(span, pos..pos)))
    }
}
