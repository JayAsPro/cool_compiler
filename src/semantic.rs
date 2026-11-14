use crate::common::*;

fn erro_semantico<T: std::fmt::Debug>(node: &T, message: &str) {
    eprintln!("{:#?}", node);
    eprintln!("ERRO SEMÂNTICO (linha ): {}.", message);
}

fn class_exists(type_id: &str, classes: &[Class]) -> bool {
    return classes.iter().any(|class| class.type_id == type_id)
}

fn class_and_parents<'a>(mut type_id: &'a str, classes: &'a [Class]) -> Vec<Environment<'a>> {
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

fn method_type(class_type_id: &str, object_id: &str, classes: &[Class]) -> String {
    let env_stack = class_and_parents(class_type_id, classes);
    
    for environment in env_stack.iter().rev() {
        match environment {
            Environment::Class(class) => {
                if let Some(method) = class.methods.iter().find(
                    |method| method.object_id == object_id
                ) {
                    return method.type_id.clone()
                }
            }

            _ => ()
        }
    }

    return String::new()
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
                if !expr.semantic(env_stack, classes) {
                    return false
                }

                if from_type_id.is_empty() {
                    *from_type_id = expr.self_type_id().clone()
                }

                *type_id = method_type(
                    fill_self_type(from_type_id, env_stack), object_id, classes
                );

                if type_id.is_empty() {
                    erro_semantico(self, "método não definido");
                    return false
                }
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
