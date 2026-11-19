use crate::common::*;
use crate::semantic::class_and_parents;

// ============================================================================
// Constantes
// ============================================================================
//
// Modelo de objetos (todo valor Cool é um i32 = endereço na memória linear;
// o endereço 0 representa "void"):
//
//   offset 0 : id da classe   (i32)  -> índice da classe em program.classes
//   offset 4 : tamanho total do objeto em bytes (i32)
//   offset 8 : atributo 0     (i32)
//   offset 12: atributo 1     (i32)
//   ...
//
// Atributos herdados vêm primeiro, na ordem da classe mais antiga para a mais
// nova. Assim, o offset de um atributo (guardado em Attribute.offset) é o mesmo
// em qualquer subclasse, e um objeto de uma subclasse pode ser usado onde se
// espera a superclasse.
//
// Int, Bool e String também são objetos (boxed), com atributos ocultos que são
// inseridos na AST no início da geração:
//   Int    -> "$val"
//   Bool   -> "$val"
//   String -> "$len" e "$ptr"
// O caractere '$' não pode aparecer em identificadores Cool, então não há
// conflito de nomes com atributos do usuário.

const WORD_SIZE: usize = 4;
const HEADER_SIZE: usize = 8;

// Os primeiros bytes da memória ficam reservados (dados estáticos, como
// literais de string, serão colocados aqui nas próximas etapas).
const STATIC_DATA_SIZE: usize = 1024;

// ============================================================================
// Preparação da AST (atributos ocultos, offsets e tamanhos)
// ============================================================================

fn count_attributes(env_stack: &Vec<Environment>) -> usize {
    let mut count = 0;

    for environment in env_stack {
        if let Environment::Class(class) = environment {
            count += class.attributes.len()
        }
    }

    return count
}

// Tamanho do objeto: cabeçalho + atributos da classe e de todas as antecessoras.
fn class_size(class: &Class, classes: &[Class]) -> usize {
    return HEADER_SIZE + WORD_SIZE * count_attributes(&class_and_parents(&class.type_id, classes))
}

fn hidden_attribute(object_id: &str) -> Attribute {
    return Attribute {
        object_id: String::from(object_id),
        type_id: String::from("$raw"),
        offset: 0,
        expr: Expr::Nil
    }
}

// Int, Bool e String não têm atributos na AST: insere os atributos ocultos.
fn add_hidden_attributes(classes: &mut [Class]) {
    for class in classes.iter_mut() {
        let names = match class.type_id.as_str() {
            "Int" | "Bool" => vec!["$val"],
            "String" => vec!["$len", "$ptr"],
            _ => Vec::new()
        };

        for (index, name) in names.into_iter().enumerate() {
            class.attributes.insert(index, hidden_attribute(name));
        }
    }
}

// O primeiro atributo de uma classe começa logo depois dos atributos das
// antecessoras. Os demais seguem em sequência.
fn set_offsets(classes: &mut [Class]) {
    let start_offsets: Vec<usize> = classes
        .iter()
        .map(|class| {
            HEADER_SIZE + WORD_SIZE * count_attributes(&class_and_parents(&class.inherits, classes))
        })
        .collect();

    for (class, mut offset) in classes.iter_mut().zip(start_offsets) {
        for attribute in class.attributes.iter_mut() {
            attribute.offset = offset as i32;
            offset += WORD_SIZE;
        }
    }
}

// ============================================================================
// Geração de código WebAssembly (formato folded)
// ============================================================================

fn indent(text: &str) -> String {
    let mut result = String::new();

    for line in text.lines() {
        if !line.is_empty() {
            result.push_str("  ");
            result.push_str(line);
        }

        result.push('\n');
    }

    return result
}

fn gen_memory() -> String {
    return format!(
        "(memory (export \"memory\") 1)\n(global $heap_ptr (mut i32) (i32.const {}))\n",
        STATIC_DATA_SIZE
    )
}

// Alocador "bump": nunca libera memória. Como a memória do WebAssembly começa
// zerada e nunca é reaproveitada, todo objeto novo tem todos os campos = 0.
fn gen_alloc() -> String {
    return String::from(r#"(func $alloc (param $size i32) (result i32)
  (local $ptr i32)
  (local $missing i32)
  (local.set $ptr (global.get $heap_ptr))
  (global.set $heap_ptr (i32.add (local.get $ptr) (local.get $size)))
  (local.set $missing
    (i32.sub (global.get $heap_ptr) (i32.mul (memory.size) (i32.const 65536))))
  (if (i32.gt_s (local.get $missing) (i32.const 0))
    (then
      (drop (memory.grow
        (i32.add (i32.div_u (local.get $missing) (i32.const 65536)) (i32.const 1))))))
  (local.get $ptr))
"#)
}

fn gen_class_ids(classes: &[Class]) -> String {
    let mut code = String::new();

    for (class_id, class) in classes.iter().enumerate() {
        code += &format!("(global $class_{} i32 (i32.const {}))\n", class.type_id, class_id);
    }

    return code
}

// $init_X inicializa os atributos de um objeto já alocado: primeiro chama o
// init da superclasse e depois inicializa os atributos declarados em X.
fn gen_init(class: &Class) -> String {
    let mut code = format!("(func $init_{} (param $self i32)\n", class.type_id);

    if !class.inherits.is_empty() {
        code += &format!("  (call $init_{} (local.get $self))\n", class.inherits);
    }

    for attribute in &class.attributes {
        // TODO (próxima etapa): gerar a expressão do atributo e guardar
        // o resultado em [self + offset].
        code += &format!(
            "  ;; {} : {} (offset {})\n",
            attribute.object_id, attribute.type_id, attribute.offset
        );
    }

    code += ")\n";
    return code
}

// $new_X aloca um objeto da classe X, preenche o cabeçalho e o inicializa.
fn gen_new(class_id: usize, class: &Class, classes: &[Class]) -> String {
    return format!(
r#"(func $new_{name} (result i32)
  (local $self i32)
  (local.set $self (call $alloc (i32.const {size})))
  (i32.store (local.get $self) (i32.const {id}))
  (i32.store offset=4 (local.get $self) (i32.const {size}))
  (call $init_{name} (local.get $self))
  (local.get $self))
"#,
        name = class.type_id,
        size = class_size(class, classes),
        id = class_id
    )
}

fn gen_classes(classes: &[Class]) -> String {
    let mut code = String::new();

    for (class_id, class) in classes.iter().enumerate() {
        code += &format!(
            ";; Classe {} (id {}, {} bytes)\n",
            class.type_id, class_id, class_size(class, classes)
        );

        code += &gen_init(class);
        code += &gen_new(class_id, class, classes);
        code += "\n";
    }

    return code
}

pub fn gen_program(program: &mut Program) -> String {
    let mut body = String::new();

    add_hidden_attributes(&mut program.classes);
    set_offsets(&mut program.classes);

    body += &gen_memory();
    body += "\n";
    body += &gen_alloc();
    body += "\n";
    body += &gen_class_ids(&program.classes);
    body += "\n";
    body += &gen_classes(&program.classes);

    return format!("(module\n{})\n", indent(&body))
}
