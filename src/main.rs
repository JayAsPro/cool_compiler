use std::env;
use std::fs;

mod ast;
mod common;
mod lexical;
mod syntax;
mod semantic;
mod generation;


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
        if state.ast.semantic() {
            let output = generation::gen_program(&mut state.ast);
            println!("{}", output);

            if let Ok(_) = fs::write("output.bril", output) {
                println!("Compilação concluída com sucesso!")
            }
        }
    }
    
    // syntax::token_loop(&code);
    // println!("{:#?}", state.ast)
}
