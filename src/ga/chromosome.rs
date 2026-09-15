use crate::{
    Config, GeneratorData,
    ga::{gene::Gene, plot::Plot},
};
use rand::{RngExt, rngs::StdRng};

#[derive(Clone)]
pub struct Chromosome {
    pub genes: Vec<Gene>,
    pub fitness: Option<i32>,
}

impl Chromosome {
    pub fn random(gens: &Vec<(GeneratorData, u8)>, rng: &mut StdRng, plot: &Plot) -> Self {
        let mut genes = Vec::new();

        for (generator, count) in gens {
            for _ in 0..*count {
                genes.push(Gene::random(generator.id, rng, &plot, &generator));
            }
        }

        Self {
            genes,
            fitness: None,
        }
    }
    pub fn mutate(&self, rng: &mut StdRng, plot: &Plot, config: &Config) -> Self {
        let mut mutant = self.clone();
        mutant.fitness = None;

        let w = plot.width;
        let h = plot.height;
        for genes in mutant.genes.iter_mut() {
            if rng.random_bool(config.mutate) {
                if rng.random_bool(config.mutate_pos) {
                    if rng.random_bool(config.mutate_pos_by_nudge) {
                        let dx: i32 = rng.random_range(-2..=2);
                        let dy: i32 = rng.random_range(-2..=2);
                        genes.x = (genes.x as i32 + dx).clamp(0, w as i32 - 1) as u8;
                        genes.y = (genes.y as i32 + dy).clamp(0, h as i32 - 1) as u8;
                    } else {
                        let (x, y) = plot.random_legal_position(rng, genes.w, genes.h);
                        genes.x = x;
                        genes.y = y;
                    }
                }
                if rng.random_bool(config.mutate_rotation) {
                    genes.rotated = !genes.rotated;
                }
                if rng.random_bool(config.mutate_placement) {
                    let was_placed = genes.placed;
                    genes.placed = !genes.placed;
                    if !was_placed && genes.placed {
                        let (x, y) = plot.random_legal_position(rng, genes.w, genes.h);
                        genes.x = x;
                        genes.y = y;
                    }
                }
            }
        }

        mutant
    }

    pub fn crossover(&self, rng: &mut StdRng, other: &Self) -> Self {
        let mut new_genes = Vec::with_capacity(self.genes.len());
        for i in 0..self.genes.len() {
            new_genes.push(if rng.random_bool(0.5) {
                self.genes[i].clone()
            } else {
                other.genes[i].clone()
            });
        }
        Self {
            genes: new_genes,
            fitness: None,
        }
    }

    pub fn repair(&mut self, plot: &Plot, rng: &mut StdRng) {
        let mut occupied = vec![false; (plot.width as usize) * (plot.height as usize)];

        let order: Vec<usize> = (0..self.genes.len()).collect();

        for &gi in &order {
            let g = &mut self.genes[gi];
            if !g.placed {
                continue;
            }

            let (w, h) = if g.rotated { (g.h, g.w) } else { (g.w, g.h) };

            if !plot.fits_cleanly(g.x, g.y, w, h, &occupied) {
                if let Some((nx, ny)) = plot.find_legal_position(rng, w, h, &occupied, 20) {
                    g.x = nx;
                    g.y = ny;
                } else {
                    g.placed = false;
                    continue;
                }
            }

            plot.mark_occupied(g.x, g.y, w, h, &mut occupied);
        }

        self.fitness = None;
    }
}
