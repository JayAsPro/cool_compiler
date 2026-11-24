// Gerado por IA: Claude Sonnet 5.5, esforço médio ou baixo.

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

// Classes cujos valores não são objetos alocados com new_<Classe>.
const PRIMITIVE_CLASSES: [&str; 3] = ["Int", "Bool", "String"];

const TYPE_INT: &str = "\"int\"";
const TYPE_BOOL: &str = "\"bool\"";
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

    // Compara dois ints; o resultado é um "bool" de Bril (não um int de Cool).
    fn compare(&mut self, op: &str, arg1: &str, arg2: &str) -> String {
        let dest = self.fresh();

        self.instrs.push(format!(
            "{{\"op\": \"{}\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\", \"{}\"]}}",
            op, dest, TYPE_BOOL, arg1, arg2
        ));

        return dest
    }

    // Copia o valor de uma variável para outra já existente (ou nova).
    fn assign(&mut self, dest: &str, source: &str) {
        self.instrs.push(format!(
            "{{\"op\": \"id\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\"]}}",
            dest, TYPE_INT, source
        ));
    }

    fn copy(&mut self, source: &str) -> String {
        let dest = self.fresh();
        self.assign(&dest, source);
        return dest
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        self.counter += 1;
        return format!("{}_{}", prefix, self.counter)
    }

    // Converte um "bool" de Bril em um Bool de Cool (int 0 ou 1).
    fn bool_to_int(&mut self, condition: &str) -> String {
        let result = self.fresh();
        let label_true = self.fresh_label("true");
        let label_false = self.fresh_label("false");
        let label_end = self.fresh_label("end");

        self.instrs.push(format!(
            "{{\"op\": \"br\", \"args\": [\"{}\"], \"labels\": [\"{}\", \"{}\"]}}",
            condition, label_true, label_false
        ));

        self.instrs.push(format!("{{\"label\": \"{}\"}}", label_true));
        self.instrs.push(format!(
            "{{\"op\": \"const\", \"dest\": \"{}\", \"type\": {}, \"value\": 1}}", result, TYPE_INT
        ));
        self.instrs.push(format!("{{\"op\": \"jmp\", \"labels\": [\"{}\"]}}", label_end));

        self.instrs.push(format!("{{\"label\": \"{}\"}}", label_false));
        self.instrs.push(format!(
            "{{\"op\": \"const\", \"dest\": \"{}\", \"type\": {}, \"value\": 0}}", result, TYPE_INT
        ));

        self.instrs.push(format!("{{\"label\": \"{}\"}}", label_end));
        return result
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
    let handle = function.load("heap");
    let next = function.binary("add", &handle, "size");
    function.store("heap", &next);
    function.ret(&handle);

    return function
}

// Função principal do programa Bril: cria o heap e instancia Main.
fn gen_main() -> Function {
    let mut function = Function::new("main", Vec::new(), None);

    let heap_size = function.const_int(HEAP_SIZE);

    function.instrs.push(format!(
        "{{\"op\": \"alloc\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\"]}}",
        "heap", TYPE_PTR, heap_size
    ));

    // heap[0] = 1 (o handle 0 é reservado para "void").
    let one = function.const_int(1);

    function.store("heap", &one);

    let object = function.call("new_Main", &["heap"]);
    function.call("Main.main", &["heap", object.as_str()]);
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

// Gera uma constante de Cool.
fn gen_constant(function: &mut Function, type_id: &str, value: &str) -> String {
    return match type_id {
        "Int" => function.const_int(value.parse().unwrap_or(0)),
        "Bool" => function.const_int(if value == "true" { 1 } else { 0 }),
        "String" => gen_string(function, value),

        // Object ou qualquer outra classe: sem inicializador, o valor é void (handle 0).
        // Como todo objeto é um handle "int", o tipo do atributo não muda o layout.
        _ => function.const_int(0)
    }
}

// Retorna os atributos de uma classe na ordem do layout: herdados primeiro.
fn class_attributes<'a>(type_id: &'a str, classes: &'a [Class]) -> Vec<&'a Attribute> {
    let mut attributes: Vec<&Attribute> = Vec::new();

    for environment in semantic::class_and_parents(type_id, classes) {
        if let Environment::Class(class) = environment {
            for attribute in &class.attributes {
                attributes.push(&attribute)
            }
        }
    }

    return attributes
}

// Contexto de geração de uma expressão: onde procurar os identificadores.
struct Context<'a> {
    attributes: Vec<&'a Attribute>,
    formals: &'a [Formal],
    self_var: String
}

fn formal_var(object_id: &str) -> String {
    return format!("arg_{}", object_id)
}

// Gera o código de uma expressão e retorna a variável Bril com o seu valor.
fn gen_expr(function: &mut Function, context: &Context, expr: &Expr) -> String {
    return match expr {
        Expr::Constant { type_id, value } => gen_constant(function, type_id, value),

        Expr::Identifier { object_id, .. } => {
            if object_id == "self" {
                context.self_var.clone()
            } else if context.formals.iter().any(|formal| formal.object_id == *object_id) {
                function.copy(&formal_var(object_id))
            } else if let Some(index) = context.attributes.iter().rposition(
                |attribute| attribute.object_id == *object_id
            ) {
                let slot = function.slot(&context.self_var, 1 + index as i64);
                function.load(&slot)
            } else {
                function.const_int(0)
            }
        }

        Expr::Assigment { object_id, expr, .. } => {
            let value = gen_expr(function, context, expr);

            if context.formals.iter().any(|formal| formal.object_id == *object_id) {
                function.assign(&formal_var(object_id), &value);
            } else if let Some(index) = context.attributes.iter().rposition(
                |attribute| attribute.object_id == *object_id
            ) {
                let slot = function.slot(&context.self_var, 1 + index as i64);
                function.store(&slot, &value);
            }

            value
        }

        Expr::New { type_id } => match type_id.as_str() {
            "Int" | "Bool" => function.const_int(0),
            "String" => gen_string(function, "\"\""),
            _ => function.call(&format!("new_{}", type_id), &["heap"])
        },

        Expr::Isvoid { expr, .. } => {
            let value = gen_expr(function, context, expr);

            // Int e Bool nunca são void; String e objetos são void se o handle for 0.
            match expr.self_type_id().as_str() {
                "Int" | "Bool" => function.const_int(0),

                _ => {
                    let zero = function.const_int(0);
                    let condition = function.compare("eq", &value, &zero);
                    function.bool_to_int(&condition)
                }
            }
        }

        Expr::Parenthesis { expr, .. } => gen_expr(function, context, expr),

        // Demais expressões: ainda não suportadas (valem void).
        _ => function.const_int(0)
    }
}

// Gera new_<Classe>(heap) -> handle, que aloca e inicializa um objeto.
// Layout: [id da classe, atributos herdados..., atributos próprios...].
fn gen_constructor(class_id: usize, class: &Class, classes: &[Class]) -> Function {
    let mut function = Function::new(
        &format!("new_{}", class.type_id), vec![("heap", TYPE_PTR)], Some(TYPE_INT)
    );

    let attributes = class_attributes(&class.type_id, classes);

    let size = function.const_int(1 + attributes.len() as i64);
    let object = call_alloc(&mut function, &size);

    let tag_slot = function.slot(&object, 0);
    let tag = function.const_int(class_id as i64);
    function.store(&tag_slot, &tag);

    let context = Context { attributes: attributes.clone(), formals: &[], self_var: object.clone() };

    for (index, attribute) in attributes.into_iter().enumerate() {
        let value = gen_expr(&mut function, &context, &attribute.expr);
        let slot = function.slot(&object, 1 + index as i64);
        function.store(&slot, &value);
    }

    function.ret(&object);
    return function
}

// Gera <Classe>.<método>(heap, self, parâmetros...) -> int.
fn gen_method(class: &Class, method: &Method, classes: &[Class]) -> Function {
    let formal_vars: Vec<String> = method.formals.iter().map(|f| formal_var(&f.object_id)).collect();
    let mut args = vec![("heap", TYPE_PTR), ("self", TYPE_INT)];

    for formal in &formal_vars {
        args.push((formal.as_str(), TYPE_INT));
    }

    let mut function = Function::new(
        &format!("{}.{}", class.type_id, method.object_id), args, Some(TYPE_INT)
    );

    let context = Context {
        attributes: class_attributes(&class.type_id, classes),
        formals: &method.formals,
        self_var: String::from("self")
    };

    let value = gen_expr(&mut function, &context, &method.expr);
    function.ret(&value);

    return function
}

pub fn gen_program(program: &Program) -> String {
    let mut functions = vec![gen_alloc_function(), gen_main()];

    for (class_id, class) in program.classes.iter().enumerate() {
        if !PRIMITIVE_CLASSES.contains(&class.type_id.as_str()) {
            functions.push(gen_constructor(class_id, class, &program.classes));
        }

        // Os métodos das classes padrão serão escritos em Bril no passo 5.
        if !BUILTIN_CLASSES.contains(&class.type_id.as_str()) {
            for method in &class.methods {
                functions.push(gen_method(class, method, &program.classes));
            }
        }
    }

    let functions: Vec<String> = functions.iter().map(|f| format!("    {}", f.to_json())).collect();
    return format!("{{\n  \"functions\": [\n{}\n  ]\n}}", functions.join(",\n"))
}
