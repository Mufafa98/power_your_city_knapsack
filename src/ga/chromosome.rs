use std::iter;

use rand::{RngExt, rngs::StdRng};

use crate::{
    GeneratorData,
    ga::{gene::Gene, plot::Plot},
};

#[derive(Clone)]
pub struct Chromosome {
    pub genes: Vec<Gene>,
    pub fitness: i32,
}

impl Chromosome {
    pub fn new(gens: &Vec<(GeneratorData, u8)>) -> Self {
        let mut genes = Vec::new();

        for (generator, count) in gens {
            genes.extend(iter::repeat(Gene::from(*generator)).take(*count as usize));
        }

        Self { genes, fitness: 0 }
    }

    pub fn mutate(&self, rng: &mut StdRng, plot: &Plot) -> Self {
        let mut mutant = self.clone();

        let w = plot.width;
        let h = plot.height;
        for genes in mutant.genes.iter_mut() {
            if rng.random_bool(0.5) {
                genes.placed = !genes.placed;
                if genes.placed && rng.random_bool(0.5) {
                    genes.x = rng.random_range(0..w);
                    genes.y = rng.random_range(0..h);
                }
            }
        }

        mutant
    }

    pub fn crossover(&self, rng: &mut StdRng, other: &Self) -> Self {
        let idx = rng.random_range(0..self.genes.len());
        let mut new_genes: Vec<Gene> = Vec::with_capacity(self.genes.len());

        let from_first = rng.random_bool(0.5);

        for i in 0..self.genes.len() {
            if from_first {
                if i < idx {
                    new_genes.push(self.genes[i].clone());
                } else {
                    new_genes.push(other.genes[i].clone());
                }
            } else {
                if i >= idx {
                    new_genes.push(self.genes[i].clone());
                } else {
                    new_genes.push(other.genes[i].clone());
                }
            }
        }

        Self {
            genes: new_genes,
            fitness: 0,
        }
    }
}
