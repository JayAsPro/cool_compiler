use crate::common::*;

fn erro_semantico<T: std::fmt::Debug>(node: &T, message: &str) {
    eprintln!("{:#?}", node);
    eprintln!("ERRO SEMÂNTICO (linha ): {}.", message);
}

fn find_class<'a>(type_id: &str, classes: &'a [Class]) -> Option<&'a Class> {
    return classes.iter().find(|class| class.type_id == type_id)
}

fn class_exists(type_id: &str, classes: &[Class]) -> bool {
    return classes.iter().any(|class| class.type_id == type_id)
}

pub fn class_and_parents<'a>(mut type_id: &'a str, classes: &'a [Class]) -> Vec<Environment<'a>> {
    let mut env_stack = Vec::new();

    while let Some(class) = classes.iter().find(|class| class.type_id == type_id) {
        env_stack.push(Environment::Class(class));
        type_id = &class.inherits
    }
    
    env_stack.reverse();
    return env_stack
}

fn fill_self_type<'a>(type_id: &'a str, env_stack: &Vec<Environment<'a>>) -> &'a str {
    if type_id == "SELF_TYPE" {
        for environment in env_stack.iter().rev() {
            if let Environment::Class(class) = environment {
                return &class.type_id
            }
        }
    }

    return type_id
}

fn nearest_parent<'a>(
    mut type_id_1: &'a str, mut type_id_2: &'a str, env_stack: &Vec<Environment<'a>>, classes: &[Class]
) -> String {
    let mut nearest_type_id = "";

    type_id_1 = fill_self_type(type_id_1, env_stack);
    type_id_2 = fill_self_type(type_id_2, env_stack);

    let env_stack_1 = class_and_parents(type_id_1, classes);
    let env_stack_2 = class_and_parents(type_id_2, classes);

    for matching_envs in env_stack_1.into_iter().zip(env_stack_2) {
        match matching_envs {
            (Environment::Class(class_1), Environment::Class(class_2)) => {
                if class_1.type_id != class_2.type_id { break }
                nearest_type_id = &class_1.type_id;
            }

            _ => break
        }
    }

    return String::from(nearest_type_id)
}

fn attribute_type(object_id: &str, env_stack: &Vec<Environment>) -> String {
    for environment in env_stack.iter().rev() {
        match environment {
            Environment::Class(class) => {
                if let Some(attribute) = class.attributes.iter().find(
                    |attribute| attribute.object_id == object_id
                ) {
                    return attribute.type_id.clone()
                }
            }

            Environment::Method(method) => {
                if let Some(formal) = method.formals.iter().find(
                    |formal| formal.object_id == object_id
                ) {
                    return formal.type_id.clone()
                }
            }

            Environment::Expr(expr) => {
                match expr {
                    Expr::Let { list, .. } => {
                        if let Some(attribute) = list.iter().find(
                            |attribute| attribute.object_id == object_id
                        ) {
                            return attribute.type_id.clone()
                        }
                    }

                    Expr::Case { list, .. } => {
                        if let Some(branch) = list.iter().find(
                            |branch| branch.object_id == object_id
                        ) {
                            return branch.type_id.clone()
                        }
                    }

                    _ => ()
                }
            }
        }
    }

    return String::new()
}

fn find_method<'a>(
    class_type_id: &'a str, object_id: &str, classes: &'a[Class], search_stack: bool
) -> Option<(&'a Class, &'a Method)> {
    if search_stack {
        let env_stack = class_and_parents(class_type_id, classes);
    
        for environment in env_stack.iter().rev() {
            match environment {
                Environment::Class(class) => {
                    if let Some(method) = class.methods.iter().find(
                        |method| method.object_id == object_id
                    ) {
                        return Some((class, method))
                    }
                }

                _ => ()
            }
        }
    } else {
        if let Some(class) = find_class(class_type_id, classes) {
            if let Some(method) = class.methods.iter().find(
                |method| method.object_id == object_id
            ) {
                return Some((class, method))
            }
        }
    }
    
    return None
}

impl Expr {
    fn semantic(&mut self, env_stack: &mut Vec<Environment>, classes: &[Class]) -> bool {
        match self {
            Expr::Identifier { type_id, object_id } => {
                if type_id.is_empty() {
                    *type_id = attribute_type(object_id, env_stack);

                    if type_id.is_empty() {
                        erro_semantico(self, "atributo não definido");
                        return false
                    }
                }
            }

            Expr::Assigment { type_id, object_id, expr } => {
                let attrib_type_id = attribute_type(object_id, env_stack);
                let nearest;

                if attrib_type_id.is_empty() {
                    erro_semantico(self, "atributo não definido");
                    return false
                }

                if !expr.semantic(env_stack, classes) {
                    return false
                }

                if type_id.is_empty() {
                    *type_id = expr.self_type_id()
                }

                nearest = nearest_parent(&attrib_type_id, type_id, env_stack, classes);

                if nearest != attrib_type_id {
                    erro_semantico(self,
                        "o tipo estático da expressão não conforma ao tipo declarado do atributo"
                    );

                    return false
                }
            }

            Expr::Dispatch {
                type_id, expr, from_type_id,
                object_id, params
            } => {
                let find_method_res;
                let expr_type_id;

                if !expr.semantic(env_stack, classes) { return false }

                expr_type_id = expr.self_type_id();

                if from_type_id.is_empty() {
                    find_method_res = find_method(
                        fill_self_type(&expr_type_id, env_stack),
                        object_id, classes, true
                    );
                    
                    if let Some((class, ..)) = find_method_res {
                        *from_type_id = class.type_id.clone()
                    }
                } else {
                    find_method_res = find_method(
                        fill_self_type(from_type_id, env_stack),
                        object_id, classes, false
                    );
                }

                if let Some((.., method)) = find_method_res {
                    *type_id = method.type_id.clone();

                    if params.len() != method.formals.len() {
                        erro_semantico(self, "número errado de parâmetros em chamada de método");
                        return false
                    }

                    if !params.iter_mut().all(|parameter| parameter.semantic(env_stack, classes)) {
                        return false
                    }

                    for (parameter, formal) in params.iter().zip(&method.formals) {
                        if nearest_parent(
                            &parameter.self_type_id(), &formal.type_id, env_stack, classes
                        ) != formal.type_id {
                            erro_semantico(parameter,
                                "tipo da expressão não conforma ao tipo do parâmetro do método"
                            );

                            return false
                        }
                    }
                } else {
                    erro_semantico(self,
                        "método não definido nesta classe ou nas suas antecessoras"
                    );

                    return false
                }
            }

            Expr::Conditional {
                type_id, expr_if,
                expr_then, expr_else
            } => {
                if !expr_if.semantic(env_stack, classes)
                || !expr_then.semantic(env_stack, classes)
                || !expr_else.semantic(env_stack, classes) {
                    return false
                }

                if expr_if.self_type_id() != "Bool" {
                    erro_semantico(expr_if, "tipo da expressão \"if\" diferente de Bool");
                    return false
                }

                *type_id = nearest_parent(
                    &expr_then.self_type_id(), &expr_else.self_type_id(),
                    env_stack, classes
                );
            }

            Expr::Loop { expr_while, expr_loop, .. } => {
                if !expr_while.semantic(env_stack, classes)
                || !expr_loop.semantic(env_stack, classes) {
                    return false
                }

                if expr_while.self_type_id() != "Bool" {
                    erro_semantico(expr_while, "tipo da expressão \"while\" diferente de Bool");
                    return false
                }
            }

            Expr::Block { type_id, list } => {
                if !list.iter_mut().all(|expr| expr.semantic(env_stack, classes)) {
                    return false
                }

                if let Some(expr) = list.last() {
                    *type_id = expr.self_type_id()
                }
            }

            Expr::Let { type_id, list, expr } => {
                
            }

            _ => return true
        }

        return true
    }
}

impl Formal {
    pub fn semantic(&self, classes: &[Class]) -> bool {
        if !class_exists(&self.type_id, classes) {
            erro_semantico(self, "tipo não definido");
            return false;
        }

        return true
    }
}

impl Attribute {
    fn without_exprs(&self) -> Attribute {
        return Attribute {
            object_id: self.object_id.clone(),
            type_id: self.type_id.clone(),
            offset: self.offset,
            expr: Expr::Nil
        }
    }

    pub fn semantic(&mut self, env_stack: &mut Vec<Environment>, classes: &[Class]) -> bool {
        let nearest;

        if self.object_id == "self" {
            erro_semantico(self, "identificador inválido para atributos (\"self\")");
            return false
        }

        if self.type_id != "SELF_TYPE" && !class_exists(&self.type_id, classes) {
            erro_semantico(self, "tipo não definido");
            return false
        }

        if !self.expr.semantic(env_stack, classes) {
            return false
        }

        nearest = nearest_parent(
            &self.type_id, &self.expr.self_type_id(), env_stack, classes
        );

        if nearest != self.type_id {
            erro_semantico(self,
                "o tipo estático da expressão não conforma ao tipo declarado do atributo"
            );

            return false
        }

        return true
    }
}

impl Method {
    fn without_exprs(&self) -> Method {
        let mut method_without_exprs = Method {
            object_id: self.object_id.clone(),
            type_id: self.type_id.clone(),
            formals: Vec::new(),
            expr: Expr::Nil
        };

        for formal in &self.formals {
            method_without_exprs.formals.push(formal.clone())
        }

        return method_without_exprs;
    }

    pub fn semantic(&mut self, env_stack: &mut Vec<Environment>, classes: &[Class]) -> bool {
        if self.object_id == "self" {
            erro_semantico(self, "identificador inválido para métodos (\"self\")");
            return false
        }

        if self.type_id != "SELF_TYPE" && !class_exists(&self.type_id, classes) {
            erro_semantico(self, "tipo não definido");
            return false
        }

        if !self.formals.iter().all(|formal| formal.semantic(classes)) {
            return false
        }

        return self.expr.semantic(env_stack, classes)
    }
}

impl Class {
    fn without_exprs(&self) -> Class {
        let mut class_without_exprs = Class {
            type_id: self.type_id.clone(),
            inherits: self.inherits.clone(),
            attributes: Vec::new(),
            methods: Vec::new()
        };

        for attribute in &self.attributes {
            class_without_exprs.attributes.push(attribute.without_exprs())
        }

        for method in &self.methods {
            class_without_exprs.methods.push(method.without_exprs())
        }

        return class_without_exprs
    }

    fn check_type_and_inheritance(&self, classes: &[Class]) -> bool {
        if classes.iter().filter(|class| class.type_id == self.type_id).count() > 1 {
            erro_semantico(self, "redefinição de tipo");
            return false;
        }

        match self.inherits.as_str() {
            "Int" | "Bool" | "String" => {
                erro_semantico(self, "este tipo não pode ser herdado");
                return false;
            }

            &_ => ()
        }

        if &self.inherits != "" && !class_exists(&self.inherits, classes) {
            erro_semantico(self, "tipo herdado não definido");
            return false;
        }

        return true
    }

    pub fn semantic(&mut self, classes: &[Class]) -> bool {
        let mut env_stack = class_and_parents(&self.type_id, classes);

        if !self.attributes.iter_mut().all(
            |attribute| attribute.semantic(&mut env_stack, classes)
        ) {
            return false
        }

        return self.methods.iter_mut().all(
            |method| method.semantic(&mut env_stack, classes)
        )
    }
}

impl Program {
    fn classes_without_exprs(&self) -> Vec<Class> {
        let mut classes = Vec::new();

        for class in &self.classes {
            classes.push(class.without_exprs())
        }

        return classes;
    }

    pub fn semantic(&mut self) -> bool {
        if let Some(main_class) = self.classes.iter().find(
            |&class| class.type_id == "Main"
        ) {
            if let Some(_) = main_class.methods.iter().find(
                |&method| method.object_id == "main"
            ) {
                if !self.classes.iter().all(
                    |class| class.check_type_and_inheritance(&self.classes)
                ) {
                    return false
                }
                
                let classes = self.classes_without_exprs();
                return self.classes.iter_mut().all(|class| class.semantic(&classes))
            } else {
                erro_semantico(self, "método main não definido na classe Main");
                return false
            }
        } else {
            erro_semantico(self, "classe Main não definida");
            return false
        }
    }
}
