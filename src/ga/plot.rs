use std::collections::HashMap;

use crate::{GeneratorData, ga::chromosome::Chromosome};

#[derive(Clone)]
pub struct Plot {
    pub subplot_size: u8,
    pub subplot_grid: (u8, u8),
    pub multipliers: Vec<u8>,

    pub width: u8,
    pub height: u8,
}

impl Plot {
    pub fn new(subplot_size: u8, subplot_grid: (u8, u8), multipliers: Vec<u8>) -> Self {
        Self {
            subplot_size,
            subplot_grid,
            multipliers,

            width: subplot_size * subplot_grid.0,
            height: subplot_size * subplot_grid.1,
        }
    }

    pub fn evaluate_chromosome(
        &self,
        chromosome: &mut Chromosome,
        generators: &[(GeneratorData, u8)],
    ) -> i32 {
        let mut fitness = 0;
        let mut occupied = vec![false; (self.width as usize) * (self.height as usize)];

        let gen_map: HashMap<u8, GeneratorData> =
            generators.iter().map(|(g, _)| (g.id, *g)).collect();

        for gene in chromosome.genes.iter() {
            if !gene.placed {
                continue;
            }
            let x = gene.x as i32;
            let y = gene.y as i32;

            let generator = gen_map.get(&gene.id).unwrap();

            let w = generator.width as i32;
            let h = generator.height as i32;

            let (w, h) = if gene.rotated { (h, w) } else { (w, h) };

            let value = generator.base_value as i32;
            let area = w * h;

            let w_in_plot = w.min(self.width as i32 - x);
            let h_in_plot = h.min(self.height as i32 - y);

            let area_in_plot = w_in_plot * h_in_plot;
            let area_out_plot = area - area_in_plot;

            let mut overlap_area = 0;
            let mut base_fitness = 0;

            for i in 0..w_in_plot {
                for j in 0..h_in_plot {
                    let cx = x + i;
                    let cy = y + j;

                    let subplot_x = cx / self.subplot_size as i32;
                    let subplot_y = cy / self.subplot_size as i32;
                    let subplot_idx = (subplot_y * self.subplot_grid.0 as i32 + subplot_x) as usize;

                    let multiplier = self.multipliers[subplot_idx] as i32;
                    base_fitness += value * multiplier;

                    let idx = ((y + j) * self.width as i32 + (x + i)) as usize;
                    if occupied[idx] {
                        overlap_area += 1;
                    }
                    occupied[idx] = true;
                }
            }

            let outside_penalty = area_out_plot * value;
            let overlap_penalty = overlap_area * value * 5;
            fitness += (base_fitness - outside_penalty - overlap_penalty) / area;
        }
        chromosome.fitness = fitness;
        fitness
    }
}
