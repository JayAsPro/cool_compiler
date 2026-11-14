use crate::common::*;

fn erro_semantico<T: std::fmt::Debug>(node: &T, message: &str) {
    eprintln!("{:#?}", node);
    eprintln!("ERRO SEMÂNTICO (linha ): {}.", message);
}

fn class_exists(type_id: &str, classes: &[Class]) -> bool {
    return classes.iter().any(|class| class.type_id == type_id)
}

fn class_and_parents<'a>(mut type_id: &str, classes: &'a [Class]) -> Vec<Environment<'a>> {
    let mut env_stack = Vec::new();

    if let Some(class) = classes.iter().find(|class| class.type_id == type_id) {
        env_stack.push(Environment::Class(&class));
        
        let mut inherits = class.inherits.as_str();

        while let Some(class) = classes.iter().find(|class| class.type_id == inherits) {
            env_stack.push(Environment::Class(&class));
            inherits = &class.inherits;
        }
    }
    
    env_stack.reverse();
    return env_stack
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

    pub fn semantic(&mut self, classes: &[Class]) -> bool {
        if self.object_id == "self" {
            erro_semantico(self, "identificador inválido para atributos (\"self\")");
            return false
        }

        if !class_exists(&self.type_id, classes) {
            erro_semantico(self, "tipo não definido");
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

    pub fn semantic(&mut self, classes: &[Class]) -> bool {
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

        return true
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
        if !self.attributes.iter_mut().all(|attribute| attribute.semantic(classes)) {
            return false
        }

        return self.methods.iter_mut().all(|method| method.semantic(classes))
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
