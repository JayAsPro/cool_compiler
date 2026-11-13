use crate::common::*;

fn erro_semantico<T: std::fmt::Debug>(node: &T, message: &str) {
    eprintln!("{:#?}", node);
    eprintln!("ERRO SEMÂNTICO (linha ): {}.", message);
}

fn type_exists(type_id: &str, classes: &[Class]) -> bool {
    return classes.iter().any(|class| class.type_id == type_id)
}

impl Formal {
    pub fn semantic(&self, classes: &[Class]) -> bool {
        if !type_exists(&self.type_id, classes) {
            erro_semantico(self, "tipo não definido");
            return false;
        }

        return true
    }
}

impl Attribute {
    pub fn semantic(&self, classes: &[Class]) -> bool {
        if self.type_id != "SELF_TYPE" && !type_exists(&self.type_id, classes) {
            erro_semantico(self, "tipo não definido");
            return false
        }

        return true
    }
}

impl Method {
    pub fn semantic(&self, classes: &[Class]) -> bool {
        if self.type_id != "SELF_TYPE" && !type_exists(&self.type_id, classes) {
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

        if &self.inherits != "" && !type_exists(&self.inherits, classes) {
            erro_semantico(self, "tipo herdado não definido");
            return false;
        }

        return true
    }

    pub fn semantic(&self, classes: &[Class]) -> bool {
        if !self.attributes.iter().all(|attribute| attribute.semantic(classes)) {
            return false
        }

        return self.methods.iter().all(|method| method.semantic(classes))
    }
}

impl Program {
    pub fn semantic(&self) -> bool {
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

                return self.classes.iter().all(|class| class.semantic(&self.classes))
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
