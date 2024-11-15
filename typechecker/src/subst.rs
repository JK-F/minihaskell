use std::collections::{HashMap, HashSet};

use ast::ast::Type;
use log::info;

use crate::{error::TypingError, util::tvars_in};

#[derive(Clone, Debug)]
pub struct Substitution {
    map: HashMap<String, Type>,
}

impl Substitution {
    pub fn extended(self, tv: String, t: Type) -> Result<Substitution, TypingError> {
        info!("Extending substitution with {} <-s> {}", tv, t);
        if let Type::TypeVariable(tv2) = &t {
            if tv.eq(tv2) {
                return Ok(self);
            }
        }
        if tvars_in(&t).contains(&&tv) {
            return Err(TypingError::DuplicateTypeVariable(tv));
        }
        let mut map = self.map;
        map.insert(tv, t);
        Ok(Substitution { map })
    }
    pub fn apply(&self, tv: &String) -> Type {
        match self.map.get(tv) {
            Some(t) => t.clone(),
            None => Type::TypeVariable(tv.clone()),
        }
    }

    fn new() -> Substitution {
        Substitution {
            map: HashMap::new(),
        }
    }

    pub fn id_subst() -> Substitution {
        Substitution::new()
    }

    pub fn from(map: HashMap<String, Type>) -> Substitution {
        if !map.is_empty() {
            info!(
                "Building substitution {{ {} }}",
                map.iter()
                    .map(|(from, to)| format!("{} <-s> {}", from, to))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        Substitution { map }
    }

    pub fn range(self) -> HashSet<String> {
        let mut res = HashSet::new();
        for (_, x) in self.map {
            match x {
                Type::TypeVariable(tvar) => {res.insert(tvar);},
                _ => unreachable!(),
            }
        }
        res
    }

    pub fn exclude(&self, scheme_vars: &HashSet<String>) -> Substitution {
        let mut map = self.map.clone();
        scheme_vars.iter().for_each(|var| {
            map.remove(var);
        });
        Substitution { map }
    }
}

pub fn subst_combine(left: Substitution, right: Substitution) -> Substitution {
    let mut map = right.map;
    for (k, v) in left.map {
        map.insert(k, v);
    }
    Substitution { map }
}
