// Gerado por IA: Claude Sonnet 5.5, esforço médio ou baixo.

use crate::common::*;
use crate::semantic;

// Representação em Bril:
//
// Todo valor de Cool (Int, Bool, String, objetos) é um "int" em Bril.
// Objetos e strings são "handles": índices dentro de um heap global, que é
// um único bloco "ptr<int>" passado como primeiro argumento de toda função.
// O handle 0 representa "void".
//
//   heap[0]       -> próximo índice livre (alocador incremental)
//   objeto        -> [id da classe, atributo 0, atributo 1, ...]
//   string        -> [código do char 0, código do char 1, ..., 0]
//   Bool          -> 0 (false) ou 1 (true)

const HEAP_SIZE: i64 = 1_000_000;

const BUILTIN_CLASSES: [&str; 5] = ["Object", "Int", "Bool", "String", "IO"];

const TYPE_INT: &str = "\"int\"";
const TYPE_PTR: &str = "{\"ptr\": \"int\"}";

struct Function {
    name: String,
    args: Vec<(String, String)>,
    ret_type: Option<String>,
    instrs: Vec<String>,
    counter: usize
}

impl Function {
    fn new(name: &str, args: Vec<(&str, &str)>, ret_type: Option<&str>) -> Function {
        return Function {
            name: String::from(name),
            args: args.into_iter().map(|(n, t)| (String::from(n), String::from(t))).collect(),
            ret_type: ret_type.map(String::from),
            instrs: Vec::new(),
            counter: 0
        }
    }

    // Gera um nome novo de variável.
    fn fresh(&mut self) -> String {
        self.counter += 1;
        return format!("v{}", self.counter)
    }

    fn const_int(&mut self, value: i64) -> String {
        let dest = self.fresh();

        self.instrs.push(format!(
            "{{\"op\": \"const\", \"dest\": \"{}\", \"type\": {}, \"value\": {}}}",
            dest, TYPE_INT, value
        ));

        return dest
    }

    fn binary(&mut self, op: &str, arg1: &str, arg2: &str) -> String {
        let dest = self.fresh();

        self.instrs.push(format!(
            "{{\"op\": \"{}\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\", \"{}\"]}}",
            op, dest, TYPE_INT, arg1, arg2
        ));

        return dest
    }

    // Retorna um ponteiro para heap[handle + offset].
    fn slot(&mut self, handle: &str, offset: i64) -> String {
        let offset_var = self.const_int(offset);
        let index = self.binary("add", handle, &offset_var);
        let dest = self.fresh();

        self.instrs.push(format!(
            "{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}",
            dest, TYPE_PTR, index
        ));

        return dest
    }

    fn store(&mut self, pointer: &str, value: &str) {
        self.instrs.push(format!(
            "{{\"op\": \"store\", \"args\": [\"{}\", \"{}\"]}}", pointer, value
        ));
    }

    fn load(&mut self, pointer: &str) -> String {
        let dest = self.fresh();

        self.instrs.push(format!(
            "{{\"op\": \"load\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\"]}}",
            dest, TYPE_INT, pointer
        ));

        return dest
    }

    fn call(&mut self, function: &str, args: &[&str]) -> String {
        let dest = self.fresh();
        let args: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();

        self.instrs.push(format!(
            "{{\"op\": \"call\", \"dest\": \"{}\", \"type\": {}, \"funcs\": [\"{}\"], \"args\": [{}]}}",
            dest, TYPE_INT, function, args.join(", ")
        ));

        return dest
    }

    fn ret(&mut self, value: &str) {
        self.instrs.push(format!("{{\"op\": \"ret\", \"args\": [\"{}\"]}}", value));
    }

    fn to_json(&self) -> String {
        let args: Vec<String> = self.args.iter().map(
            |(name, tk_type)| format!("{{\"name\": \"{}\", \"type\": {}}}", name, tk_type)
        ).collect();

        let ret_type = match &self.ret_type {
            Some(tk_type) => format!("\"type\": {}, ", tk_type),
            None => String::new()
        };

        return format!(
            "{{\"name\": \"{}\", \"args\": [{}], {}\"instrs\": [\n        {}\n    ]}}",
            self.name, args.join(", "), ret_type, self.instrs.join(",\n        ")
        )
    }
}

// Aloca "size" posições no heap e retorna o handle.
fn call_alloc(function: &mut Function, size: &str) -> String {
    return function.call("alloc", &["heap", size])
}

// Função auxiliar: alloc(heap, size) -> handle (alocador incremental).
fn gen_alloc_function() -> Function {
    let mut function = Function::new(
        "alloc", vec![("heap", TYPE_PTR), ("size", TYPE_INT)], Some(TYPE_INT)
    );

    // heap[0] guarda o próximo índice livre.
    let zero = function.const_int(0);
    let counter_ptr = function.fresh();

    function.instrs.push(format!(
        "{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}",
        counter_ptr, TYPE_PTR, zero
    ));

    let handle = function.load(&counter_ptr);
    let next = function.binary("add", &handle, "size");
    function.store(&counter_ptr, &next);
    function.ret(&handle);

    return function
}

// Função principal do programa Bril: cria o heap e instancia Main.
fn gen_main() -> Function {
    let mut function = Function::new("main", Vec::new(), None);

    let heap_size = function.const_int(HEAP_SIZE);
    let heap = function.fresh();

    function.instrs.push(format!(
        "{{\"op\": \"alloc\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\"]}}",
        heap, TYPE_PTR, heap_size
    ));

    // O nome "heap" é o que as demais funções esperam.
    function.instrs.push(format!(
        "{{\"op\": \"id\", \"dest\": \"heap\", \"type\": {}, \"args\": [\"{}\"]}}",
        TYPE_PTR, heap
    ));

    // heap[0] = 1 (o handle 0 é reservado para "void").
    let zero = function.const_int(0);
    let one = function.const_int(1);
    let counter_ptr = function.fresh();

    function.instrs.push(format!(
        "{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}",
        counter_ptr, TYPE_PTR, zero
    ));

    function.store(&counter_ptr, &one);

    function.call("new_Main", &["heap"]);
    function.instrs.push(String::from("{\"op\": \"free\", \"args\": [\"heap\"]}"));

    return function
}

// Gera um handle para uma string a partir do texto do token (com aspas).
fn gen_string(function: &mut Function, text: &str) -> String {
    let inner = if text.len() >= 2 { &text[1..text.len() - 1] } else { "" };
    let size = function.const_int(inner.len() as i64 + 1);
    let handle = call_alloc(function, &size);

    for (index, byte) in inner.bytes().enumerate() {
        let slot = function.slot(&handle, index as i64);
        let value = function.const_int(byte as i64);
        function.store(&slot, &value);
    }

    // Terminador.
    let slot = function.slot(&handle, inner.len() as i64);
    let zero = function.const_int(0);
    function.store(&slot, &zero);

    return handle
}

// Gera o valor inicial de um atributo (neste passo, sempre uma constante).
fn gen_attribute_value(function: &mut Function, attribute: &Attribute) -> String {
    return match &attribute.expr {
        Expr::Constant { type_id, value } => match type_id.as_str() {
            "Int" => function.const_int(value.parse().unwrap_or(0)),
            "Bool" => function.const_int(if value == "true" { 1 } else { 0 }),
            "String" => gen_string(function, value),

            // Object ou qualquer outra classe: sem inicializador, o valor é void (handle 0).
            // Como todo objeto é um handle "int", o tipo do atributo não muda o layout.
            _ => function.const_int(0)
        },

        _ => function.const_int(0)
    }
}

// Gera new_<Classe>(heap) -> handle, que aloca e inicializa um objeto.
// Layout: [id da classe, atributos herdados..., atributos próprios...].
fn gen_constructor(class_id: usize, class: &Class, classes: &[Class]) -> Function {
    let mut function = Function::new(
        &format!("new_{}", class.type_id), vec![("heap", TYPE_PTR)], Some(TYPE_INT)
    );

    let mut attributes: Vec<&Attribute> = Vec::new();
    
    for environment in semantic::class_and_parents(&class.type_id, classes) {
        if let Environment::Class(class) = environment {
            for attribute in &class.attributes {
                attributes.push(&attribute)
            }
        }
    }

    let size = function.const_int(1 + attributes.len() as i64);
    let object = call_alloc(&mut function, &size);

    let tag_slot = function.slot(&object, 0);
    let tag = function.const_int(class_id as i64);
    function.store(&tag_slot, &tag);

    for (index, attribute) in attributes.into_iter().enumerate() {
        let value = gen_attribute_value(&mut function, attribute);
        let slot = function.slot(&object, 1 + index as i64);
        function.store(&slot, &value);
    }

    function.ret(&object);
    return function
}

pub fn gen_program(program: &Program) -> String {
    let mut functions = vec![gen_alloc_function(), gen_main()];

    for (class_id, class) in program.classes.iter().enumerate() {
        if !BUILTIN_CLASSES.contains(&class.type_id.as_str()) {
            functions.push(gen_constructor(class_id, class, &program.classes));
        }
    }

    let functions: Vec<String> = functions.iter().map(|f| format!("    {}", f.to_json())).collect();
    return format!("{{\n  \"functions\": [\n{}\n  ]\n}}", functions.join(",\n"))
}
