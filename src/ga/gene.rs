use rand::{RngExt, rngs::StdRng};

use crate::{GeneratorData, ga::Plot};

#[derive(Clone)]
pub struct Gene {
    pub id: u8,
    pub x: u8,
    pub y: u8,
    pub w: u8,
    pub h: u8,
    pub value: u32,
    pub gold: bool,
    pub rotated: bool,
    pub placed: bool,
}

impl Gene {
    pub fn random(id: u8, rng: &mut StdRng, plot: &Plot, generator: &GeneratorData) -> Self {
        let (x, y) = plot.random_legal_position(rng, generator.width_, generator.height_);
        Self {
            id,
            x,
            y,
            w: generator.width_,
            h: generator.height_,
            gold: generator.gold_,
            value: generator.base_value_,
            rotated: rng.random_bool(0.5),
            placed: rng.random_bool(0.5),
        }
    }
}
