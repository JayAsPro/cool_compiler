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
//   Int    -> "$val"            (offset 8)
//   Bool   -> "$val"            (offset 8)
//   String -> "$len" e "$ptr"   (offsets 8 e 12; $ptr aponta para os bytes)
// O caractere '$' não pode aparecer em identificadores Cool, então não há
// conflito de nomes com atributos do usuário.

const WORD_SIZE: usize = 4;
const HEADER_SIZE: usize = 8;

// Offsets dos atributos ocultos (devem coincidir com add_hidden_attributes).
const INT_VALUE_OFFSET: usize = HEADER_SIZE;
const STRING_LENGTH_OFFSET: usize = HEADER_SIZE;
const STRING_POINTER_OFFSET: usize = HEADER_SIZE + WORD_SIZE;
const STRING_OBJECT_SIZE: usize = HEADER_SIZE + 2 * WORD_SIZE;

// Dados estáticos no início da memória:
//   1. textos fixos usados pelos métodos das classes padrão;
//   2. tabela com o endereço do objeto String que contém o nome de cada classe
//      (indexada pelo id da classe), seguida dos nomes e desses objetos String.
// A heap começa logo depois dos dados estáticos.
const ABORT_MESSAGE: &str = "Abort chamado a partir da classe ";
const SUBSTR_MESSAGE: &str = "ERRO: String.substr fora dos limites do array\n";
const NEWLINE: &str = "\n";

const ABORT_MESSAGE_ADDR: usize = 0;
const SUBSTR_MESSAGE_ADDR: usize = ABORT_MESSAGE_ADDR + ABORT_MESSAGE.len();
const NEWLINE_ADDR: usize = SUBSTR_MESSAGE_ADDR + SUBSTR_MESSAGE.len();
const NAME_TABLE_ADDR: usize = (NEWLINE_ADDR + NEWLINE.len() + 3) & !3;

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
// Dados estáticos
// ============================================================================

fn gen_data(address: usize, bytes: &[u8]) -> String {
    let mut text = String::new();

    for byte in bytes {
        text += &format!("\\{:02x}", byte);
    }

    return format!("(data (i32.const {}) \"{}\")\n", address, text)
}

// Palavras de 32 bits em little-endian.
fn gen_data_words(address: usize, words: &[usize]) -> String {
    let mut bytes = Vec::new();

    for word in words {
        bytes.extend_from_slice(&(*word as u32).to_le_bytes());
    }

    return gen_data(address, &bytes)
}

// Retorna o código dos dados estáticos e o endereço onde a heap deve começar.
fn gen_static_data(classes: &[Class]) -> (String, usize) {
    let mut code = String::new();

    // A classe String sempre existe (ver Program::default).
    let string_class_id = classes.iter().position(|class| class.type_id == "String").unwrap();
    let mut next_address = NAME_TABLE_ADDR + WORD_SIZE * classes.len();

    code += &gen_data(ABORT_MESSAGE_ADDR, ABORT_MESSAGE.as_bytes());
    code += &gen_data(SUBSTR_MESSAGE_ADDR, SUBSTR_MESSAGE.as_bytes());
    code += &gen_data(NEWLINE_ADDR, NEWLINE.as_bytes());

    for (class_id, class) in classes.iter().enumerate() {
        let name = class.type_id.as_bytes();
        let name_address = next_address;
        let object_address = (name_address + name.len() + 3) & !3;

        code += &gen_data(name_address, name);

        // Objeto String estático: [id da classe][tamanho][$len][$ptr].
        code += &gen_data_words(
            object_address,
            &[string_class_id, STRING_OBJECT_SIZE, name.len(), name_address]
        );

        code += &gen_data_words(NAME_TABLE_ADDR + WORD_SIZE * class_id, &[object_address]);

        next_address = object_address + STRING_OBJECT_SIZE;
    }

    return (code, next_address)
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

// Funções fornecidas pelo JavaScript (módulo "env"), que imprimem no console:
//   out_string(ptr, len)  : imprime o texto de len bytes que está em ptr, na
//                           memória exportada "memory" (o JavaScript precisa
//                           decodificar os bytes, pois o WebAssembly só passa
//                           números);
//   out_int(valor)        : imprime o inteiro (pode ser o próprio console.log).
// Importações devem vir antes de qualquer outra definição do módulo.
fn gen_imports() -> String {
    return String::from(r#"(import "env" "out_string" (func $host_out_string (param i32 i32)))
(import "env" "out_int" (func $host_out_int (param i32)))
"#)
}

fn gen_memory(heap_start: usize) -> String {
    return format!(
        "(memory (export \"memory\") 1)\n(global $heap_ptr (mut i32) (i32.const {}))\n",
        heap_start
    )
}

// Alocador "bump": nunca libera memória. O tamanho é arredondado para múltiplo
// de 4 (strings podem ter tamanho qualquer). Como a memória do WebAssembly
// começa zerada e nunca é reaproveitada, todo objeto novo tem campos = 0.
fn gen_alloc() -> String {
    return String::from(r#"(func $alloc (param $size i32) (result i32)
  (local $ptr i32)
  (local $missing i32)
  (local.set $size (i32.and (i32.add (local.get $size) (i32.const 3)) (i32.const -4)))
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

// ============================================================================
// Métodos das classes padrão (hardcoded)
// ============================================================================
//
// Convenção de nomes: $Classe.metodo. Todo método recebe $self como primeiro
// parâmetro, seguido dos parâmetros formais, e retorna um i32. Os parâmetros
// Int são objetos Int (boxed), e não números.

// Funções auxiliares para criar objetos Int e String e imprimir uma String.
fn gen_builtin_helpers() -> String {
    return format!(
r#";; Cria um objeto Int com o valor dado.
(func $new_int (param $value i32) (result i32)
  (local $int i32)
  (local.set $int (call $new_Int))
  (i32.store offset={val} (local.get $int) (local.get $value))
  (local.get $int))

;; Cria um objeto String que usa os len bytes que estão em ptr (sem copiá-los).
(func $new_string (param $ptr i32) (param $len i32) (result i32)
  (local $string i32)
  (local.set $string (call $new_String))
  (i32.store offset={len} (local.get $string) (local.get $len))
  (i32.store offset={ptr} (local.get $string) (local.get $ptr))
  (local.get $string))

;; Imprime o conteúdo de um objeto String.
(func $print_string (param $string i32)
  (call $host_out_string
    (i32.load offset={ptr} (local.get $string))
    (i32.load offset={len} (local.get $string))))
"#,
        val = INT_VALUE_OFFSET,
        len = STRING_LENGTH_OFFSET,
        ptr = STRING_POINTER_OFFSET
    )
}

fn gen_object_methods() -> String {
    return format!(
r#";; Object.abort: imprime a mensagem e interrompe a execução.
(func $Object.abort (param $self i32) (result i32)
  (call $host_out_string (i32.const {abort_addr}) (i32.const {abort_len}))
  (call $print_string (call $Object.type_name (local.get $self)))
  (call $host_out_string (i32.const {newline_addr}) (i32.const {newline_len}))
  (unreachable))

;; Object.type_name: nome da classe do objeto (o tipo dinâmico), lido na tabela de nomes.
(func $Object.type_name (param $self i32) (result i32)
  (i32.load
    (i32.add
      (i32.const {table})
      (i32.mul (i32.load (local.get $self)) (i32.const {word})))))

;; Object.copy: cópia rasa do objeto (os atributos são copiados, não os objetos apontados).
(func $Object.copy (param $self i32) (result i32)
  (local $size i32)
  (local $copy i32)
  (local.set $size (i32.load offset=4 (local.get $self)))
  (local.set $copy (call $alloc (local.get $size)))
  (memory.copy (local.get $copy) (local.get $self) (local.get $size))
  (local.get $copy))
"#,
        abort_addr = ABORT_MESSAGE_ADDR,
        abort_len = ABORT_MESSAGE.len(),
        newline_addr = NEWLINE_ADDR,
        newline_len = NEWLINE.len(),
        table = NAME_TABLE_ADDR,
        word = WORD_SIZE
    )
}

// O nome do primeiro método é "lenght" porque é assim que ele está na AST.
fn gen_string_methods() -> String {
    return format!(
r#";; String.lenght
(func $String.lenght (param $self i32) (result i32)
  (call $new_int (i32.load offset={len} (local.get $self))))

;; String.concat: aloca os bytes da nova string e copia as duas strings.
(func $String.concat (param $self i32) (param $s i32) (result i32)
  (local $len1 i32)
  (local $len2 i32)
  (local $ptr i32)
  (local.set $len1 (i32.load offset={len} (local.get $self)))
  (local.set $len2 (i32.load offset={len} (local.get $s)))
  (local.set $ptr (call $alloc (i32.add (local.get $len1) (local.get $len2))))
  (memory.copy
    (local.get $ptr)
    (i32.load offset={ptr} (local.get $self))
    (local.get $len1))
  (memory.copy
    (i32.add (local.get $ptr) (local.get $len1))
    (i32.load offset={ptr} (local.get $s))
    (local.get $len2))
  (call $new_string (local.get $ptr) (i32.add (local.get $len1) (local.get $len2))))

;; String.substr: i e l são objetos Int. Como as strings são imutáveis, a
;; substring reaproveita os bytes da string original.
(func $String.substr (param $self i32) (param $i i32) (param $l i32) (result i32)
  (local $start i32)
  (local $count i32)
  (local.set $start (i32.load offset={val} (local.get $i)))
  (local.set $count (i32.load offset={val} (local.get $l)))
  (if (i32.or
        (i32.or
          (i32.lt_s (local.get $start) (i32.const 0))
          (i32.lt_s (local.get $count) (i32.const 0)))
        (i32.gt_s
          (i32.add (local.get $start) (local.get $count))
          (i32.load offset={len} (local.get $self))))
    (then
      (call $host_out_string (i32.const {msg_addr}) (i32.const {msg_len}))
      (unreachable)))
  (call $new_string
    (i32.add (i32.load offset={ptr} (local.get $self)) (local.get $start))
    (local.get $count)))
"#,
        val = INT_VALUE_OFFSET,
        len = STRING_LENGTH_OFFSET,
        ptr = STRING_POINTER_OFFSET,
        msg_addr = SUBSTR_MESSAGE_ADDR,
        msg_len = SUBSTR_MESSAGE.len()
    )
}

fn gen_io_methods() -> String {
    return format!(
r#";; IO.out_string
(func $IO.out_string (param $self i32) (param $x i32) (result i32)
  (call $print_string (local.get $x))
  (local.get $self))

;; IO.out_int: x é um objeto Int.
(func $IO.out_int (param $self i32) (param $x i32) (result i32)
  (call $host_out_int (i32.load offset={val} (local.get $x)))
  (local.get $self))

;; IO.in_string: pede o tamanho da linha ao JavaScript, aloca os bytes e pede a cópia.
(func $IO.in_string (param $self i32) (result i32)
  (local $len i32)
  (local $ptr i32)
  (local.set $len (call $host_in_string_length))
  (local.set $ptr (call $alloc (local.get $len)))
  (call $host_in_string_read (local.get $ptr))
  (call $new_string (local.get $ptr) (local.get $len)))

;; IO.in_int
(func $IO.in_int (param $self i32) (result i32)
  (call $new_int (call $host_in_int)))
"#,
        val = INT_VALUE_OFFSET
    )
}

// Int e Bool não têm métodos próprios.
fn gen_builtin_methods() -> String {
    let mut code = String::new();

    code += &gen_builtin_helpers();
    code += "\n";
    code += &gen_object_methods();
    code += "\n";
    code += &gen_string_methods();
    code += "\n";
    code += &gen_io_methods();

    return code
}

pub fn gen_program(program: &mut Program) -> String {
    let mut body = String::new();

    add_hidden_attributes(&mut program.classes);
    set_offsets(&mut program.classes);

    let (static_data, heap_start) = gen_static_data(&program.classes);

    body += &gen_imports();
    body += "\n";
    body += &gen_memory(heap_start);
    body += &static_data;
    body += "\n";
    body += &gen_alloc();
    body += "\n";
    body += &gen_class_ids(&program.classes);
    body += "\n";
    body += &gen_classes(&program.classes);
    body += &gen_builtin_methods();

    return format!("(module\n{})\n", indent(&body))
}