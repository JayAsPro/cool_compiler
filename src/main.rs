use std::env;
use std::fs;

mod common;
mod lexical;
mod syntax;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    let Option::Some(file_path) = arguments.get(1) else {
        eprintln!("ERRO: arquivo não informado!");
        return;
    };
    
    let Result::Ok(code) = fs::read_to_string(file_path) else {
        eprintln!("ERRO: falha na abertura do arquivo!");
        return;
    };

    // syntax_analysis(&code);
    syntax::program(&code);
}
