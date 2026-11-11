use std::env;
use std::fs;

mod common;
mod lexical;
mod ast;
mod semantic;
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
    
    let mut state = common::State::new(&code);

    if syntax::program(&mut state) {
        state.ast.semantic();
    }
    
    // syntax::token_loop(&code);
}
