use crate::common::*;

fn erro_semantico<T: std::fmt::Debug>(node: &T, message: &str) {
    eprintln!("{:#?}", node);
    eprintln!("ERRO SEMÂNTICO (linha ): {}.", message);
}

impl Program {
    fn type_and_parents<'a>(&'a self, mut type_id: &'a str) -> Vec<&'a str> {
        let mut type_ids: Vec<&'a str> = Vec::new();
    
        for class in self.classes.iter().rev() {
            if class.type_id == type_id {
                type_ids.push(&class.type_id);
                type_id = &class.inherits;
            }
        }
    
        return type_ids;
    }
    
    fn type_exists<'a>(&self, type_id: &'a str) -> bool {
        return !self.type_and_parents(type_id).is_empty()
    }

    pub fn semantic(&self) -> bool {
        if let Some(main_class) = self.classes.iter().find(
            |&class| class.type_id == "Main"
        ) {
            if let Some(_) = main_class.methods.iter().find(
                |&method| method.object_id == "main"
            ) {
                return true
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

impl Formal {
    pub fn semantic(&self, program: &Program) -> bool {
        if !program.type_exists(&self.type_id) {
            erro_semantico(self, "tipo não definido");
            return false;
        }

        return true
    }
}

impl Attribute {
    pub fn semantic(&self, program: &Program) -> bool {
        if !program.type_exists(&self.type_id) {
            erro_semantico(self, "tipo não definido");
            return false
        }

        return true
    }
}

impl Method {
    pub fn semantic(&self, program: &Program) -> bool {
        if !program.type_exists(&self.type_id) {
            erro_semantico(self, "tipo não definido");
            return false
        }

        return true
    }
}

impl Class {
    pub fn semantic(&self, program: &Program) -> bool {
        if program.type_exists(&self.type_id) {
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

        if !program.type_exists(&self.inherits) {
            erro_semantico(self, "tipo herdado não definido");
            return false;
        }

        return true
    }
}
