# cool_compiler: Compilador de Cool para WebAssembly escrito em Rust

## Etapa de geração de código

cool_compiler é um compilador em desenvolvimento para a linguagem de programação Cool (Classroom Object-Oriented Language), escrito em Rust. Atualmente, ele possui um analisador léxico e um analisador sintático funcionais, e um analisador semântico incompleto.

Sua tarefa é escrever a etapa de geração de código do compilador. A linguagem alvo escolhida é WebAssembly.

Você deve criar um novo módulo, chamado generation.rs, que ficará na raiz do crate, como todos os outros módulos do projeto. Ele deverá exportar uma função chamada gen_program(...), que receberá como parâmetro a raiz da árvore sintática abstrata (AST) criada a partir do código original, que é uma struct Program. Com o auxílio de funções auxiliares, ela deve gerar o código equivalente em WebAssembly, em formato textual.

Considere que o módulo WebAssembly será executado pelo navegador. Use as APIs de JavaScript do WebAssembly para imprimir as saídas (out_string e out_int) diretamente no body da página HTML e para receber entradas (in_string e in_int) de um campo de entrada de texto no HTML.

Quanto ao formato textual das instruções, escolha o que for mais simples de implementar: o folded (utilizando S-expressions) ou o formato regular.

### Regras a serem seguidas

1. **NÃO** olhe nem copie código de outros compiladores nem materiais com licenças não livres (como livros): use seu conhecimento geral sobre construção de compiladores.
2. **NÃO** pesquise a documentação da linguagem Cool na Internet.
3. **NÃO** escreva ou sugira código para nenhum dos outros módulos, a não ser que seja necessário para a geração de código: você só escreverá código para o arquivo generation.rs. Considere que as etapas anteriores estão finalizadas.
4. Mantenha o seu código simples e fácil de entender: não use macros, não tente fazer um compilador otimizador, não tente ser "clever" demais.
5. **NÃO** gere código JavaScript ou HTML se eu não pedir.
6. Construa o código de generation.rs de forma iterativa. A cada passo, escreva apenas a lógica necessária para gerar um módulo WebAssembly funcional que atinja o objetivo.
7. Escreva o seu código na ordem "dependências - dependentes". Declare todas as novas structs e enums criados no início do arquivo.
8. A implementação dos métodos das classes padrão de Cool (Object, Int, Bool e String) não existe na AST: implemente, de forma *hardcoded*, estes métodos diretamente em WebAssembly.

### EXEMPLO DE AST GERADA PELO cool_compiler

#### Código

class A {
    num : Int <- 10;
    
    number() : Int {
        num
    };
};

class Main inherits IO {
    a : A <- new A;
    
    main() : Object {
        out_int(a.number())
    };
};

#### AST

Program {
    classes: [
        Class {
            type_id: "Object",
            inherits: "",
            attributes: [],
            methods: [
                Method {
                    object_id: "abort",
                    type_id: "Object",
                    formals: [],
                    expr: Nil,
                },
                Method {
                    object_id: "type_name",
                    type_id: "String",
                    formals: [],
                    expr: Nil,
                },
                Method {
                    object_id: "copy",
                    type_id: "SELF_TYPE",
                    formals: [],
                    expr: Nil,
                },
            ],
        },
        Class {
            type_id: "Int",
            inherits: "Object",
            attributes: [],
            methods: [],
        },
        Class {
            type_id: "Bool",
            inherits: "Object",
            attributes: [],
            methods: [],
        },
        Class {
            type_id: "String",
            inherits: "Object",
            attributes: [],
            methods: [
                Method {
                    object_id: "lenght",
                    type_id: "Int",
                    formals: [],
                    expr: Nil,
                },
                Method {
                    object_id: "concat",
                    type_id: "String",
                    formals: [
                        Formal {
                            object_id: "s",
                            type_id: "String",
                        },
                    ],
                    expr: Nil,
                },
                Method {
                    object_id: "substr",
                    type_id: "SELF_TYPE",
                    formals: [
                        Formal {
                            object_id: "i",
                            type_id: "Int",
                        },
                        Formal {
                            object_id: "l",
                            type_id: "Int",
                        },
                    ],
                    expr: Nil,
                },
            ],
        },
        Class {
            type_id: "IO",
            inherits: "Object",
            attributes: [],
            methods: [
                Method {
                    object_id: "out_string",
                    type_id: "SELF_TYPE",
                    formals: [
                        Formal {
                            object_id: "x",
                            type_id: "String",
                        },
                    ],
                    expr: Nil,
                },
                Method {
                    object_id: "out_int",
                    type_id: "SELF_TYPE",
                    formals: [
                        Formal {
                            object_id: "x",
                            type_id: "Int",
                        },
                    ],
                    expr: Nil,
                },
                Method {
                    object_id: "in_string",
                    type_id: "String",
                    formals: [],
                    expr: Nil,
                },
                Method {
                    object_id: "in_int",
                    type_id: "Int",
                    formals: [],
                    expr: Nil,
                },
            ],
        },
        Class {
            type_id: "A",
            inherits: "Object",
            attributes: [
                Attribute {
                    object_id: "num",
                    type_id: "Int",
                    expr: Constant {
                        type_id: "Int",
                        value: "10",
                    },
                },
            ],
            methods: [
                Method {
                    object_id: "number",
                    type_id: "Int",
                    formals: [],
                    expr: Identifier {
                        type_id: "Int",
                        object_id: "num",
                    },
                },
            ],
        },
        Class {
            type_id: "Main",
            inherits: "IO",
            attributes: [
                Attribute {
                    object_id: "a",
                    type_id: "A",
                    expr: New {
                        type_id: "A",
                    },
                },
            ],
            methods: [
                Method {
                    object_id: "main",
                    type_id: "Object",
                    formals: [],
                    expr: Dispatch {
                        type_id: "SELF_TYPE",
                        expr: Identifier {
                            type_id: "SELF_TYPE",
                            object_id: "self",
                        },
                        from_type_id: "SELF_TYPE",
                        object_id: "out_int",
                        params: [
                            Dispatch {
                                type_id: "Int",
                                expr: Identifier {
                                    type_id: "A",
                                    object_id: "a",
                                },
                                from_type_id: "A",
                                object_id: "number",
                                params: [],
                            },
                        ],
                    },
                },
            ],
        },
    ],
}