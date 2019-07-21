extern crate lazy_static;
extern crate regex;
extern crate log;

use log::debug;
mod input;
mod lex;
mod parse;
mod render;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // args
    if args.len() <= 0 {
        return;
    }
    let filename: &str = &args[0];

    let src = input::read_file(filename);

    let mut lexer = lex::Lexer::new(src);
    lexer.tokenize();
    let tokens = lexer.get_tokens();
    debug!("Getting tokens done!");
    // for token in tokens {
    //     eprintln!("{:?}, {}", token, lexer.token_source(token));
    // }
    let mut parser = parse::Parser::new(tokens);
    let nodes = parser.parse();
    debug!("-------------- generate HTML! ---------------");
    let html = render::render(nodes);
    println!("{}", html);
}
