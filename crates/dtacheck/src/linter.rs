use std::collections::HashMap;
use std::ops::Range;

use arson_parse::reporting as codespan_reporting;
use arson_parse::Expression;
use arson_parse::ExpressionValue;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;

pub trait Lint {
    fn to_codespan(&self, id: usize) -> Diagnostic<usize>;
}

impl<'src> Lint for arson_parse::Diagnostic {
    fn to_codespan(&self, id: usize) -> Diagnostic<usize> {
        self.to_codespan(id)
    }
}

pub fn lint_file(ast: &[Expression], funcs: &Function) -> Vec<Box<dyn Lint>> {
    let mut lints = Vec::new();
    lint_node(&mut lints, ast, funcs);
    lints
}

fn lint_node(
    lints: &mut Vec<Box<dyn Lint>>,
    ast: &[Expression],
    funcs: &Function,
) {
    for node in ast {
        match &node.value {
            ExpressionValue::Array(array)
            | ExpressionValue::Property(array) => {
                lint_node(lints, &array, funcs)
            }
            ExpressionValue::Define(_, array) => {
                lint_node(lints, &array.exprs, funcs)
            }
            ExpressionValue::Command(array) => {
                lint_node(lints, array, funcs);

                let has_preprocessor_directive = array.iter().any(|e| {
                    matches!(e.value, ExpressionValue::Conditional { .. })
                });

                if !has_preprocessor_directive {
                    lint_fn_args(lints, array, node.location.clone(), funcs);
                    /*
                    lint_switch_fallthrough(
                        lints,
                        array,
                        node.location.clone(),
                    );
                    */
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
    pub fn lookup(&self, stmt: &[Expression]) -> (&Function, usize) {
        self.lookup_inner(stmt, 0)
    }

    fn lookup_inner(
        &self,
        stmt: &[Expression],
        depth: usize,
    ) -> (&Function, usize) {
        if self.children.is_empty() {
            return (self, depth);
        };

        let Some(node) = stmt.first() else {
            return (self, depth);
        };

        let ExpressionValue::Symbol(sym) = node.value else {
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
    stmt: &[Expression],
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

fn generate_function_name(stmt: &[Expression]) -> String {
    let list: Vec<&str> = stmt
        .iter()
        .map(|x| match x.value {
            ExpressionValue::Symbol(sym) => Some(sym),
            _ => None,
        })
        .take_while(Option::is_some)
        .map(|x| x.unwrap())
        .collect();

    list.join(" ")
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
    stmt: &[Expression],
    span: Range<usize>,
) {
    if stmt.is_empty() {
        return;
    }

    let ExpressionValue::Symbol(sym) = stmt[0].value else {
        return;
    };

    if sym != "switch" {
        return;
    }

    let Some(last_node) = stmt.last() else {
        return;
    };

    if matches!(last_node.value, ExpressionValue::Array(_)) {
        let pos = span.end - 1;
        lints.push(Box::new(SwitchFallthroughLint(span, pos..pos)))
    }
}
