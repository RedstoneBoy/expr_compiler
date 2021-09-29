use expr_compiler::{parser::Parser, scanner::{Error, Scanner}, token::Token};

fn main() {
    let scanner = Scanner::new("5 + 5 * 10 / $a > a($b = if (true) 13 else { $c = 10; $c })");
    let (tokens, errors): (Vec<Token>, Vec<Error>) = scanner
        .scan()
        .into_iter()
        .fold((Vec::new(), Vec::new()), |(mut toks, mut errs), err| match err {
            Ok(tok) => { toks.push(tok); (toks, errs) }
            Err(err) => { errs.push(err); (toks, errs) }
        });
    if errors.len() > 0 {
        println!("{:?}", errors);
        std::process::exit(1);
    }
    let parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();
    println!("{:?}", ast);
}