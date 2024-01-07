// This is an annotated version of swap_art_bytes that tries to explain some
// bits of rust that may not be clear for those coming from dta.

// `use <item>` to bring items into scope as an example, `use std::vec::Vec`
// will make `std::vec::Vec` available as `Vec`
use std::error::Error;
use std::path::Path;

use clap::Parser;

// The `derive` attribute is used to automatically implement traits on an item.
// Traits are simply interfaces that can be implemented by structs and enums.
//
// In this case, the `Parser` trait from `clap` will be automatically
// implemented for `Args`
//
// see: https://doc.rust-lang.org/book/ch10-02-traits.html
//      https://doc.rust-lang.org/reference/attributes/derive.html
#[derive(clap::Parser)]
// Args is a struct containing two items.
struct Args {
    // A `Box<T>` is a smart pointer to a heap allocated instance of type T.
    // Pointers are values containing the address of another value in memory.
    // They effectively *point* at another value much like a sign.
    //
    // A smart pointer takes it one step further by automatically managing
    // the resources used by the pointed to value.
    //
    // A Path is a type for dealing with filesystem paths.
    //
    // input_file and output_file are both a Box<T> containing a Path
    input_file: Box<Path>,
    output_file: Box<Path>
}
// `fn funcname(arg1: type, arg2: type) -> returntype` is used to declare a
// function. Note: if a function could fail (e.g. via invalid parameters), you
// should use Result<returntype, Error> and let the caller handle it.
//
// Result wraps an `Ok` value and an `Err` value into one type.
fn main() -> Result<(), Box<dyn Error>> {
    // `let` is used to declare a constant variable, `let mut` is used to
    // declare a non-constant (i.e. mutable) variable.
    //
    // This statement calls `clap::Parser::parse()` on `Args` to get the input arguments
    let args = Args::parse();

    // the `?` operator here is used to "unwrap" a result, returning an error to the
    // calling function if appropriate.
    //
    // `?` may also be used on Option<T> as well.
    let mut buf = std::fs::read(args.input_file)?;


    // `(32..buf.len()) creates an iterator that returns a series of `usize`
    // values going from 32 up to but not including `buf.len()` Calling
    // `step_by(n: usize)` on this iterator returns a new iterator that returns
    // every nth value in this iterator.
    //
    // For loops themselves operate on iterators.
    //
    // see: https://doc.rust-lang.org/book/ch03-05-control-flow.html
    //      https://doc.rust-lang.org/book/ch13-02-iterators.html
    for i in (32..buf.len()).step_by(2) {
        let byte1 = buf[i];
        let byte2 = buf[i + 1];

        buf[i] = byte2;
        buf[i + 1] = byte1;
    }

    std::fs::write(args.output_file, buf)?;

    // Return an empty tuple as the `Ok` value.
    Ok(())
}
