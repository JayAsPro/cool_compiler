// Quase totalmente gerado por IA:
// Claude Sonnet 5.5, com esforço médio ou baixo, e GPT 5.6 Luna, sem recurso "Pensar".

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
const TYPE_CHAR: &str = "\"char\"";

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
    fn const_bool(&mut self, value: bool) -> String {
        let dest = self.fresh();
        self.instrs.push(format!(
            "{{\"op\": \"const\", \"dest\": \"{}\", \"type\": {}, \"value\": {}}}",
            dest, TYPE_BOOL, value
        ));
        return dest
    }

    fn compare(&mut self, op: &str, arg1: &str, arg2: &str) -> String {
        let dest = self.fresh();

        self.instrs.push(format!(
            "{{\"op\": \"{}\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\", \"{}\"]}}",
            op, dest, TYPE_BOOL, arg1, arg2
        ));

        return dest
    }

    // Copia o valor de uma variável para outra já existente (ou nova).
    fn logic_or(&mut self, arg1: &str, arg2: &str) -> String {
        let dest = self.fresh();
        self.instrs.push(format!(
            "{{\"op\": \"or\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\", \"{}\"]}}",
            dest, TYPE_BOOL, arg1, arg2
        ));
        return dest
    }

    fn bool_not(&mut self, value: &str) -> String {
        let dest = self.fresh();
        self.instrs.push(format!(
            "{{\"op\": \"not\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\"]}}",
            dest, TYPE_BOOL, value
        ));
        return dest
    }

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

    fn label(&mut self, name: &str) {
        self.instrs.push(format!("{{\"label\": \"{}\"}}", name));
    }

    fn jump(&mut self, label: &str) {
        self.instrs.push(format!("{{\"op\": \"jmp\", \"labels\": [\"{}\"]}}", label));
    }

    fn branch(&mut self, condition: &str, yes: &str, no: &str) {
        self.instrs.push(format!(
            "{{\"op\": \"br\", \"args\": [\"{}\"], \"labels\": [\"{}\", \"{}\"]}}",
            condition, yes, no
        ));
    }

    fn print(&mut self, value: &str) {
        self.instrs.push(format!("{{\"op\": \"print\", \"args\": [\"{}\"]}}", value));
    }

    fn int_to_char(&mut self, value: &str) -> String {
        let dest = self.fresh();
        self.instrs.push(format!(
            "{{\"op\": \"int2char\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"{}\"]}}",
            dest, TYPE_CHAR, value
        ));
        dest
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
    self_var: String,
    current_class: &'a str,
    classes: &'a [Class],
    locals: Vec<(String, String)>
}

fn formal_var(object_id: &str) -> String {
    return format!("arg_{}", object_id)
}

fn local_index(context: &Context, object_id: &str) -> Option<usize> {
    context.locals.iter().rposition(|(name, _)| name == object_id)
}

fn method_owner(type_id: &str, method_id: &str, classes: &[Class]) -> String {
    for environment in semantic::class_and_parents(type_id, classes) {
        if let Environment::Class(class) = environment {
            if class.methods.iter().any(|method| method.object_id == method_id) {
                return class.type_id.clone();
            }
        }
    }
    return type_id.to_string()
}

fn gen_expr(function: &mut Function, context: &Context, expr: &Expr) -> String {
    return match expr {
        Expr::Constant { type_id, value } => gen_constant(function, type_id, value),

        Expr::Identifier { object_id, .. } => {
            if object_id == "self" {
                context.self_var.clone()
            } else if let Some(index) = local_index(context, object_id) {
                function.copy(&context.locals[index].1)
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
            if let Some(index) = local_index(context, object_id) {
                function.assign(&context.locals[index].1, &value);
            } else if context.formals.iter().any(|formal| formal.object_id == *object_id) {
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

        Expr::UnaryOperator { op, expr, .. } => {
            let value = gen_expr(function, context, expr);
            match op.as_str() {
                "~" => {
                    let zero = function.const_int(0);
                    function.binary("sub", &zero, &value)
                }
                "not" => {
                    let zero = function.const_int(0);
                    let condition = function.compare("eq", &value, &zero);
                    function.bool_to_int(&condition)
                }
                _ => function.const_int(0)
            }
        }

        Expr::BinaryOperator { expr1, op, expr2, .. } => {
            let left = gen_expr(function, context, expr1);
            let right = gen_expr(function, context, expr2);
            match op.as_str() {
                "+" => function.binary("add", &left, &right),
                "-" => function.binary("sub", &left, &right),
                "*" => function.binary("mul", &left, &right),
                "/" => function.binary("div", &left, &right),
                "<" => {
                    let condition = function.compare("lt", &left, &right);
                    function.bool_to_int(&condition)
                }
                "<=" => {
                    let condition = function.compare("le", &left, &right);
                    function.bool_to_int(&condition)
                }
                "=" if expr1.self_type_id() == "String" => function.call("string_equal", &["heap", &left, &right]),
                "=" => {
                    let condition = function.compare("eq", &left, &right);
                    function.bool_to_int(&condition)
                }
                _ => function.const_int(0)
            }
        }

        Expr::Dispatch { expr, from_type_id, object_id, params, .. } => {
            let receiver = gen_expr(function, context, expr);
            let static_type = if !from_type_id.is_empty() {
                from_type_id.clone()
            } else {
                let inferred = expr.self_type_id();
                if inferred == "SELF_TYPE" { String::from(context.current_class) } else { inferred }
            };
            let mut values = vec![String::from("heap"), receiver.clone()];
            for param in params { values.push(gen_expr(function, context, param)); }
            let refs: Vec<&str> = values.iter().map(|value| value.as_str()).collect();

            // Dispatch comum é dinâmico: escolhe a implementação de acordo com
            // a tag da classe concreta. Dispatch estático (@Tipo) usa o tipo pedido.
            if !from_type_id.is_empty() || PRIMITIVE_CLASSES.contains(&static_type.as_str()) {
                if object_id == "copy" && PRIMITIVE_CLASSES.contains(&static_type.as_str()) {
                    receiver
                } else if object_id == "type_name" && PRIMITIVE_CLASSES.contains(&static_type.as_str()) {
                    gen_string(function, &format!("\"{}\"", static_type))
                } else {
                    let owner = method_owner(&static_type, object_id, context.classes);
                    function.call(&format!("{}.{}", owner, object_id), &refs)
                }
            } else {
                let tag_pointer = function.slot(&receiver, 0);
                let tag = function.load(&tag_pointer);
                let result = function.fresh();
                let end = function.fresh_label("dispatch_end");
                let mut candidates = Vec::new();
                for (id, class) in context.classes.iter().enumerate() {
                    if PRIMITIVE_CLASSES.contains(&class.type_id.as_str()) { continue; }
                    let mut is_subtype = false;
                    for environment in semantic::class_and_parents(&class.type_id, context.classes) {
                        if let Environment::Class(parent) = environment {
                            if parent.type_id == static_type { is_subtype = true; break; }
                        }
                    }
                    if is_subtype && class.methods.iter().any(|m| m.object_id == *object_id)
                        || (is_subtype && method_owner(&class.type_id, object_id, context.classes) != class.type_id)
                    {
                        candidates.push((id, class.type_id.clone()));
                    }
                }
                for (id, runtime_type) in candidates {
                    let expected = function.const_int(id as i64);
                    let matches = function.compare("eq", &tag, &expected);
                    let call_label = function.fresh_label("dispatch_call");
                    let next_label = function.fresh_label("dispatch_next");
                    function.branch(&matches, &call_label, &next_label);
                    function.label(&call_label);
                    let owner = method_owner(&runtime_type, object_id, context.classes);
                    let call_value = function.call(&format!("{}.{}", owner, object_id), &refs);
                    function.assign(&result, &call_value);
                    function.jump(&end);
                    function.label(&next_label);
                }
                let fallback_owner = method_owner(&static_type, object_id, context.classes);
                let fallback = function.call(&format!("{}.{}", fallback_owner, object_id), &refs);
                function.assign(&result, &fallback);
                function.label(&end);
                result
            }
        }

        Expr::Conditional { expr_if, expr_then, expr_else, .. } => {
            let condition_value = gen_expr(function, context, expr_if);
            let zero = function.const_int(0);
            let equal_zero = function.compare("eq", &condition_value, &zero);
            let condition = function.bool_not(&equal_zero);
            let label_then = function.fresh_label("if_then");
            let label_else = function.fresh_label("if_else");
            let label_end = function.fresh_label("if_end");
            let result = function.fresh();
            function.branch(&condition, &label_then, &label_else);
            function.label(&label_then);
            let then_value = gen_expr(function, context, expr_then);
            function.assign(&result, &then_value);
            function.jump(&label_end);
            function.label(&label_else);
            let else_value = gen_expr(function, context, expr_else);
            function.assign(&result, &else_value);
            function.label(&label_end);
            result
        }

        Expr::Loop { expr_while, expr_loop, .. } => {
            let label_test = function.fresh_label("while_test");
            let label_body = function.fresh_label("while_body");
            let label_end = function.fresh_label("while_end");
            function.jump(&label_test);
            function.label(&label_test);
            let condition_value = gen_expr(function, context, expr_while);
            let zero = function.const_int(0);
            let equal_zero = function.compare("eq", &condition_value, &zero);
            let condition = function.bool_not(&equal_zero);
            function.branch(&condition, &label_body, &label_end);
            function.label(&label_body);
            gen_expr(function, context, expr_loop);
            function.jump(&label_test);
            function.label(&label_end);
            function.const_int(0)
        }

        Expr::Block { list, .. } => {
            let mut value = function.const_int(0);
            for item in list { value = gen_expr(function, context, item); }
            value
        }

        Expr::Let { list, expr, .. } => {
            // A cópia do contexto mantém o escopo lexical sem alterar os chamadores.
            let mut nested = Context {
                attributes: context.attributes.clone(), formals: context.formals,
                self_var: context.self_var.clone(), current_class: context.current_class,
                classes: context.classes, locals: context.locals.clone()
            };
            for attribute in list {
                let initial = gen_expr(function, &nested, &attribute.expr);
                let variable = function.fresh();
                function.assign(&variable, &initial);
                nested.locals.push((attribute.object_id.clone(), variable));
            }
            let value = gen_expr(function, &nested, expr);
            value
        }

        Expr::Case { case, list, .. } => {
            let value = gen_expr(function, context, case);
            let case_type = case.self_type_id();
            let primitive = PRIMITIVE_CLASSES.contains(&case_type.as_str());
            let zero = function.const_int(0);
            let is_void = if primitive && case_type != "String" {
                function.const_bool(false)
            } else {
                function.compare("eq", &value, &zero)
            };
            let label_error = function.fresh_label("case_void");
            let label_end = function.fresh_label("case_end");
            let result = function.fresh();
            let label_first = function.fresh_label("case_check");
            function.branch(&is_void, &label_error, &label_first);
            function.label(&label_error);
            // COOL gera erro em case sobre void; sem runtime de exceções, retorna void.
            function.assign(&result, &zero);
            function.jump(&label_end);
            let mut check_label = label_first.clone();
            for (index, branch) in list.iter().enumerate() {
                function.label(&check_label);
                let condition = if primitive {
                    let matches = branch.type_id == case_type || branch.type_id == "Object";
                    function.const_bool(matches)
                } else {
                    let slot = function.slot(&value, 0);
                    let tag = function.load(&slot);
                    let mut matching_ids = Vec::new();
                    for (id, class) in context.classes.iter().enumerate() {
                        for environment in semantic::class_and_parents(&class.type_id, context.classes) {
                            if let Environment::Class(parent) = environment {
                                if parent.type_id == branch.type_id { matching_ids.push(id as i64); break; }
                            }
                        }
                    }
                    let mut condition = function.const_bool(false);
                    for id in matching_ids {
                        let expected = function.const_int(id);
                        let equal = function.compare("eq", &tag, &expected);
                        condition = function.logic_or(&condition, &equal);
                    }
                    condition
                };
                let label_match = function.fresh_label("case_match");
                let label_next = function.fresh_label("case_next");
                function.branch(&condition, &label_match, &label_next);
                function.label(&label_match);
                let mut nested = Context {
                    attributes: context.attributes.clone(), formals: context.formals,
                    self_var: context.self_var.clone(), current_class: context.current_class,
                    classes: context.classes, locals: context.locals.clone()
                };
                nested.locals.push((branch.object_id.clone(), value.clone()));
                let branch_value = gen_expr(function, &nested, &branch.expr);
                function.assign(&result, &branch_value);
                function.jump(&label_end);
                check_label = label_next;
                if index + 1 == list.len() {
                    function.label(&check_label);
                    function.assign(&result, &zero);
                    function.jump(&label_end);
                }
            }
            if list.is_empty() {
                function.label(&label_first);
                function.assign(&result, &zero);
                function.jump(&label_end);
            }
            function.label(&label_end);
            result
        }

        Expr::Nil => function.const_int(0)
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

    let context = Context {
        attributes: attributes.clone(), formals: &[], self_var: object.clone(),
        current_class: &class.type_id, classes, locals: Vec::new()
    };

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
        self_var: String::from("self"), current_class: &class.type_id,
        classes, locals: Vec::new()
    };

    let value = gen_expr(&mut function, &context, &method.expr);
    function.ret(&value);

    return function
}

// Igualdade por conteúdo para String; objetos continuam usando igualdade de identidade.
fn gen_string_equal_function() -> Function {
    let mut function = Function::new(
        "string_equal", vec![("heap", TYPE_PTR), ("a", TYPE_INT), ("b", TYPE_INT)], Some(TYPE_INT)
    );
    let zero = function.const_int(0);
    let one = function.const_int(1);
    let result = function.fresh();
    let test = function.fresh_label("string_eq_test");
    let compare = function.fresh_label("string_eq_compare");
    let next = function.fresh_label("string_eq_next");
    let yes = function.fresh_label("string_eq_yes");
    let no = function.fresh_label("string_eq_no");
    let end = function.fresh_label("string_eq_end");

    let same_handle = function.compare("eq", "a", "b");
    let check_a = function.fresh_label("string_eq_check_a");
    function.branch(&same_handle, &yes, &check_a);
    function.label(&check_a);
    let a_is_void = function.compare("eq", "a", &zero);
    let check_b = function.fresh_label("string_eq_check_b");
    function.branch(&a_is_void, &no, &check_b);
    function.label(&check_b);
    let b_is_void = function.compare("eq", "b", &zero);
    function.branch(&b_is_void, &no, &test);

    let index = function.fresh();
    function.assign(&index, &zero);
    function.label(&test);
    let address_a = function.binary("add", "a", &index);
    let address_b = function.binary("add", "b", &index);
    let pointer_a = function.fresh();
    let pointer_b = function.fresh();
    function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", pointer_a, TYPE_PTR, address_a));
    function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", pointer_b, TYPE_PTR, address_b));
    let char_a = function.load(&pointer_a);
    let char_b = function.load(&pointer_b);
    let equal = function.compare("eq", &char_a, &char_b);
    function.branch(&equal, &compare, &no);
    function.label(&compare);
    let terminator = function.compare("eq", &char_a, &zero);
    function.branch(&terminator, &yes, &next);
    function.label(&next);
    let one_for_index = function.const_int(1);
    let next_index = function.binary("add", &index, &one_for_index);
    function.assign(&index, &next_index);
    function.jump(&test);
    function.label(&yes);
    function.assign(&result, &one);
    function.jump(&end);
    function.label(&no);
    function.assign(&result, &zero);
    function.label(&end);
    function.ret(&result);
    function
}

// Métodos das classes padrão: implementados diretamente em Bril, pois suas
// implementações não fazem parte da AST do programa fonte.
fn gen_builtin_method(class: &Class, method: &Method, classes: &[Class]) -> Function {
    let mut args = vec![("heap", TYPE_PTR), ("self", TYPE_INT)];
    let formal_names: Vec<String> = method.formals.iter().map(|f| formal_var(&f.object_id)).collect();
    for name in &formal_names { args.push((name.as_str(), TYPE_INT)); }
    let mut function = Function::new(
        &format!("{}.{}", class.type_id, method.object_id), args, Some(TYPE_INT)
    );

    match (class.type_id.as_str(), method.object_id.as_str()) {
        ("Object", "abort") => {
            let ret = function.const_int(-1);
            function.ret(&ret);
        }
        ("Object", "copy") => {
            let end = function.fresh_label("copy_end");
            let tag_slot = function.slot("self", 0);
            let tag = function.load(&tag_slot);
            let result = function.fresh();
            for (id, target) in classes.iter().enumerate() {
                if PRIMITIVE_CLASSES.contains(&target.type_id.as_str()) { continue; }
                let expected = function.const_int(id as i64);
                let matches = function.compare("eq", &tag, &expected);
                let label = function.fresh_label("copy_class");
                let next = function.fresh_label("copy_next");
                function.branch(&matches, &label, &next);
                function.label(&label);
                let attrs = class_attributes(&target.type_id, classes);
                let size = function.const_int((1 + attrs.len()) as i64);
                let object = call_alloc(&mut function, &size);
                for offset in 0..=attrs.len() {
                    let source = function.slot("self", offset as i64);
                    let value = function.load(&source);
                    let destination = function.slot(&object, offset as i64);
                    function.store(&destination, &value);
                }
                function.assign(&result, &object);
                function.jump(&end);
                function.label(&next);
            }
            function.assign(&result, "self");
            function.label(&end);
            function.ret(&result);
        }
        ("Object", "type_name") => {
            let tag_slot = function.slot("self", 0);
            let tag = function.load(&tag_slot);
            let result = function.fresh();
            let end = function.fresh_label("typename_end");
            for (id, target) in classes.iter().enumerate() {
                let expected = function.const_int(id as i64);
                let matches = function.compare("eq", &tag, &expected);
                let label = function.fresh_label("typename_class");
                let next = function.fresh_label("typename_next");
                function.branch(&matches, &label, &next);
                function.label(&label);
                let value = gen_string(&mut function, &format!("\"{}\"", target.type_id));
                function.assign(&result, &value);
                function.jump(&end);
                function.label(&next);
            }
            let fallback = gen_string(&mut function, "\"Object\"");
            function.assign(&result, &fallback);
            function.label(&end);
            function.ret(&result);
        }
        ("String", "lenght") => {
            let zero = function.const_int(0);
            let one = function.const_int(1);
            let index = function.fresh();
            function.assign(&index, &zero);
            let loop_label = function.fresh_label("strlen_loop");
            let body_label = function.fresh_label("strlen_body");
            let end_label = function.fresh_label("strlen_end");
            function.label(&loop_label);
            let offset = function.binary("add", "self", &index);
            let char_ptr = function.fresh();
            function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", char_ptr, TYPE_PTR, offset));
            let character = function.load(&char_ptr);
            let is_end = function.compare("eq", &character, &zero);
            let not_end = function.bool_not(&is_end);
            function.branch(&not_end, &body_label, &end_label);
            function.label(&body_label);
            let next = function.binary("add", &index, &one);
            function.assign(&index, &next);
            function.jump(&loop_label);
            function.label(&end_label);
            function.ret(&index);
        }
        ("String", "concat") => {
            let other = formal_names.first().map(|s| s.as_str()).unwrap_or("arg_s");
            let len_self = gen_string_length_call(&mut function, "self");
            let len_other = gen_string_length_call(&mut function, other);
            let total = function.binary("add", &len_self, &len_other);
            let one = function.const_int(1);
            let size = function.binary("add", &total, &one);
            let output = call_alloc(&mut function, &size);
            let zero = function.const_int(0);
            gen_copy_string_loop(&mut function, "self", &output, &zero, &len_self);
            gen_copy_string_loop(&mut function, other, &output, &len_self, &len_other);
            let end_index = function.binary("add", &len_self, &len_other);
            let end_address = function.binary("add", &output, &end_index);
            let ptr = function.fresh();
            function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", ptr, TYPE_PTR, end_address));
            function.store(&ptr, &zero);
            function.ret(&output);
        }
        ("String", "substr") => {
            let start = formal_names.get(0).map(|s| s.as_str()).unwrap_or("arg_i");
            let length = formal_names.get(1).map(|s| s.as_str()).unwrap_or("arg_l");
            let one = function.const_int(1);
            let size = function.binary("add", length, &one);
            let output = call_alloc(&mut function, &size);
            let zero = function.const_int(0);
            let index = function.fresh();
            function.assign(&index, &zero);
            let test = function.fresh_label("substr_test");
            let body = function.fresh_label("substr_body");
            let end = function.fresh_label("substr_end");
            function.label(&test);
            let condition = function.compare("lt", &index, length);
            function.branch(&condition, &body, &end);
            function.label(&body);
            let source_index = function.binary("add", start, &index);
            let source_address = function.binary("add", "self", &source_index);
            let source_ptr = function.fresh();
            function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", source_ptr, TYPE_PTR, source_address));
            let character = function.load(&source_ptr);
            let destination_address = function.binary("add", &output, &index);
            let dest_ptr = function.fresh();
            function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", dest_ptr, TYPE_PTR, destination_address));
            function.store(&dest_ptr, &character);
            let next = function.binary("add", &index, &one);
            function.assign(&index, &next);
            function.jump(&test);
            function.label(&end);
            let terminator_address = function.binary("add", &output, &index);
            let terminator = function.fresh();
            function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", terminator, TYPE_PTR, terminator_address));
            function.store(&terminator, &zero);
            function.ret(&output);
        }
        ("IO", "out_int") => {
            if let Some(value) = formal_names.first() { function.print(value); }
            function.ret("self");
        }
        ("IO", "out_string") => {
            if let Some(value) = formal_names.first() {
                let zero = function.const_int(0);
                let one = function.const_int(1);
                let index = function.fresh();
                function.assign(&index, &zero);
                let test = function.fresh_label("printstr_test");
                let body = function.fresh_label("printstr_body");
                let end = function.fresh_label("printstr_end");
                function.label(&test);
                let address = function.binary("add", value, &index);
                let pointer = function.fresh();
                function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", pointer, TYPE_PTR, address));
                let character = function.load(&pointer);
                let is_end = function.compare("eq", &character, &zero);
                let not_end = function.bool_not(&is_end);
                function.branch(&not_end, &body, &end);
                function.label(&body);
                let printable = function.int_to_char(&character);
                function.print(&printable);
                let next = function.binary("add", &index, &one);
                function.assign(&index, &next);
                function.jump(&test);
                function.label(&end);
            }
            function.ret("self");
        }
        ("IO", "in_int") => {
            let zero = function.const_int(0);
            function.ret(&zero);
        }
        ("IO", "in_string") => {
            let empty = gen_string(&mut function, "\"\"");
            function.ret(&empty);
        }
        _ => {
            let zero = function.const_int(0);
            function.ret(&zero);
        }
    }
    return function
}

// Helpers para as rotinas de strings implementadas com memória e laços Bril.
fn gen_string_length_call(function: &mut Function, handle: &str) -> String {
    let zero = function.const_int(0);
    let one = function.const_int(1);
    let index = function.fresh();
    function.assign(&index, &zero);
    let test = function.fresh_label("str_len_test");
    let body = function.fresh_label("str_len_body");
    let end = function.fresh_label("str_len_end");
    function.label(&test);
    let address = function.binary("add", handle, &index);
    let pointer = function.fresh();
    function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", pointer, TYPE_PTR, address));
    let character = function.load(&pointer);
    let is_end = function.compare("eq", &character, &zero);
    let not_end = function.bool_not(&is_end);
    function.branch(&not_end, &body, &end);
    function.label(&body);
    let next = function.binary("add", &index, &one);
    function.assign(&index, &next);
    function.jump(&test);
    function.label(&end);
    index
}

fn gen_copy_string_loop(function: &mut Function, source: &str, destination: &str, start: &str, length: &str) {
    let zero = function.const_int(0);
    let one = function.const_int(1);
    let index = function.fresh();
    function.assign(&index, &zero);
    let test = function.fresh_label("str_copy_test");
    let body = function.fresh_label("str_copy_body");
    let end = function.fresh_label("str_copy_end");
    function.label(&test);
    let condition = function.compare("lt", &index, length);
    function.branch(&condition, &body, &end);
    function.label(&body);
    let source_offset = function.binary("add", start, &index);
    let source_address = function.binary("add", source, &source_offset);
    let source_pointer = function.fresh();
    function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", source_pointer, TYPE_PTR, source_address));
    let character = function.load(&source_pointer);
    let destination_address = function.binary("add", destination, &source_offset);
    let destination_pointer = function.fresh();
    function.instrs.push(format!("{{\"op\": \"ptradd\", \"dest\": \"{}\", \"type\": {}, \"args\": [\"heap\", \"{}\"]}}", destination_pointer, TYPE_PTR, destination_address));
    function.store(&destination_pointer, &character);
    let next = function.binary("add", &index, &one);
    function.assign(&index, &next);
    function.jump(&test);
    function.label(&end);
}

pub fn gen_program(program: &Program) -> String {
    let mut functions = vec![gen_alloc_function(), gen_string_equal_function(), gen_main()];

    for (class_id, class) in program.classes.iter().enumerate() {
        if !PRIMITIVE_CLASSES.contains(&class.type_id.as_str()) {
            functions.push(gen_constructor(class_id, class, &program.classes));
        }

        if BUILTIN_CLASSES.contains(&class.type_id.as_str()) {
            for method in &class.methods {
                functions.push(gen_builtin_method(class, method, &program.classes));
            }
        } else {
            for method in &class.methods {
                functions.push(gen_method(class, method, &program.classes));
            }
        }
    }

    let functions: Vec<String> = functions.iter().map(|f| format!("    {}", f.to_json())).collect();
    return format!("{{\n  \"functions\": [\n{}\n  ]\n}}", functions.join(",\n"))
}
