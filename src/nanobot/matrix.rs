use std;

use super::model::*;
use super::prelude::*;

pub struct Matrix {
    pub r: usize,
    pub full: Vec<bool>,
}

impl Matrix {
    pub fn new(model: &Model) -> Matrix {
        let r = model.r;
        match model.id {
            ModelId::Assemble(_) => Matrix {
                r,
                full: vec![false; r * r * r],
            },
            ModelId::Disassemble(_) => {
                let mut full = vec![false; r * r * r];
                for c in &model.targets {
                    full[c.to_linear_index(r)] = true;
                }
                Matrix { r, full }
            }
        }
    }

    pub fn fill(&mut self, c: Cord) {
        // 1. void -> full
        debug_assert!(!self.full[c.to_linear_index(self.r)]);
        self.full[c.to_linear_index(self.r)] = true;
    }

    pub fn void(&mut self, c: Cord) {
        // 1. full -> void
        debug_assert!(self.full[c.to_linear_index(self.r)]);
        self.full[c.to_linear_index(self.r)] = false;
    }
}

impl std::ops::Index<Cord> for Matrix {
    type Output = bool;
    fn index(&self, index: Cord) -> &bool {
        &self.full[index.to_linear_index(self.r)]
    }
}
