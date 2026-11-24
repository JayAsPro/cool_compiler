use crate::common::*;

const TAB_SPACES: usize = 4;

fn indent(level: usize) -> String {
    return " ".repeat(TAB_SPACES * level)
}

pub fn gen_program(program: &Program) -> String {
    let mut output = String::from("{\n");

    output.push_str(&(indent(1) + "\"functions\": [\n"));
    output.push_str(&(indent(1) + "]\n}"));

    return output
}
