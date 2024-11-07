use std::fs;
use std::path::Path;
use std::path::PathBuf;

use arson::parse::lexer;
use arson::parse::parser;
use clap::Parser as ClapParser;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::term::termcolor::StandardStream;
use codespan_reporting::term::Chars;
use dtacheck::linter::lint_file;
use dtacheck::linter::Function;
use dtacheck::linter::Lint;

#[derive(ClapParser)]
struct Args {
    file: PathBuf,
    config: PathBuf,
}

fn load_funcs(path: &Path) -> Function {
    let file_contents = fs::read_to_string(path).unwrap();
    let mut tree = Function::default();

    for line in file_contents.lines() {
        if line.starts_with('#') {
            continue;
        }

        let tokens = line.split_whitespace().collect::<Vec<_>>();
        let len = tokens.len();
        if len < 3 {
            continue;
        }

        let max_args = tokens[len - 1].parse::<usize>().unwrap();
        let min_args = tokens[len - 2].parse::<usize>().unwrap();

        tree.insert(&tokens[0..len - 2], min_args, max_args);
    }

    tree
}

fn main() {
    let args = Args::parse();
    let file_contents = &fs::read(&args.file).unwrap();
    let data = String::from_utf8_lossy(file_contents).clone();
    let funcs = load_funcs(&args.config);

    let mut files = SimpleFiles::new();
    let file_id = files.add(args.file.to_str().unwrap(), &data);

    let tokens = lexer::lex(&data);
    let (ast, diagnostics) = match parser::parse(tokens) {
        Ok(ast) => (ast, Vec::new()),
        Err(errors) => (Vec::new(), errors),
    };

    let writer = StandardStream::stderr(ColorChoice::Auto);
    let config = codespan_reporting::term::Config {
        chars: Chars::ascii(),
        ..Default::default()
    };

    for diag in diagnostics {
        let _ = term::emit(
            &mut writer.lock(),
            &config,
            &files,
            &diag.to_codespan(file_id),
        );
    }

    for diag in lint_file(&ast, &funcs) {
        let _ = term::emit(
            &mut writer.lock(),
            &config,
            &files,
            &diag.to_codespan(file_id),
        );
    }
}
