# cool_compiler: Compilador de Cool para Bril escrito em Rust

## Etapa de geração de código

cool_compiler é um compilador em desenvolvimento para a linguagem de programação Cool (Classroom Object-Oriented Language), escrito em Rust. Atualmente, ele possui um analisador léxico e um analisador sintático funcionais, um analisador semântico incompleto e um gerador de código que mal começou.

Sua tarefa é concluir a etapa de geração de código do compilador, terminando o módulo `generation`. A linguagem alvo escolhida é Bril (documentação disponível em <https://capra.cs.cornell.edu/bril/intro.html>).

A geração de código fica no módulo `generation` (em generation.rs). A função `gen_program(...)` é a única função que deve ser exportada pelo módulo: ela recebe como parâmetro uma referência para a raiz da árvore sintática abstrata (AST) criada a partir do código em Cool, e deve gerar, em uma String, um programa em Bril no formato JSON, utilizando funções auxiliares.

### Regras a serem seguidas

1. **NÃO** olhe nem copie código de outros compiladores nem materiais com licenças não livres (como livros): use seu conhecimento geral sobre construção de compiladores.
2. **NÃO** pesquise a documentação da linguagem Cool na Internet: tente inferir o máximo possível a partir do código do compilador e dos códigos de exemplo.
3. **NÃO** escreva ou sugira código para nenhum dos outros módulos, a princípio: procure escrever código apenas no arquivo `generation.rs`. a não ser que seja necessário para a geração de código. Considere que as etapas anteriores estão finalizadas.
4. Mantenha o seu código simples e fácil de entender: não use macros, não tente fazer um compilador otimizador, não tente ser "clever" demais.
5. Construa o código de `generation.rs` de forma iterativa. A cada passo, escreva apenas a lógica necessária para gerar um código Bril funcional que atinja o objetivo.
6. Escreva o código em Rust na ordem "dependências - dependentes". Defina todas as novas structs e enums que forem necessárias no início do arquivo.
7. A implementação dos métodos das classes padrão de Cool (Object, Int, Bool e String) não existe na AST: implemente, de forma *hardcoded*, estes métodos diretamente em Bril.
8. 

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
                        from_type_id: "IO",
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
