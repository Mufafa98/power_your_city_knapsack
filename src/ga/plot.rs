use rand::{RngExt, rngs::StdRng};

use crate::{Config, ga::chromosome::Chromosome};

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

    pub fn evaluate_chromosome(&self, chromosome: &mut Chromosome, config: &Config) -> i32 {
        let mut fitness = 0;
        let mut occupied = vec![false; (self.width as usize) * (self.height as usize)];

        for gene in chromosome.genes.iter() {
            if !gene.placed {
                continue;
            }
            let x = gene.x as i32;
            let y = gene.y as i32;

            let w = gene.w as i32;
            let h = gene.h as i32;
            let (w, h) = if gene.rotated { (h, w) } else { (w, h) };

            let value = gene.value as i32;
            let area = w * h;

            let mut area_out_plot = 0;
            let mut overlap_area = 0;
            let mut max_multiplier: i32 = 0;

            let mut good_cells: i32 = 0;

            for i in 0..w {
                for j in 0..h {
                    let cx = x + i;
                    let cy = y + j;

                    if cx < 0 || cy < 0 || cx >= self.width as i32 || cy >= self.height as i32 {
                        area_out_plot += 1;
                        continue;
                    }

                    let subplot_x = cx / self.subplot_size as i32;
                    let subplot_y = cy / self.subplot_size as i32;
                    let subplot_idx = (subplot_y * self.subplot_grid.0 as i32 + subplot_x) as usize;
                    let m = self.multipliers[subplot_idx] as i32;

                    if m == 0 {
                        area_out_plot += 1;
                        continue;
                    }

                    if m > 0 {
                        good_cells += 1;
                        if m > max_multiplier {
                            max_multiplier = m;
                        }
                    }

                    let idx = (cy * self.width as i32 + cx) as usize;
                    if occupied[idx] {
                        overlap_area += 1;
                    }
                    occupied[idx] = true;
                }
            }

            let tiebreak = value * good_cells / 2;

            let gold_factor = if gene.gold { 2 } else { 1 };
            let base_fitness = value * gold_factor * max_multiplier;

            let penalty_mult = max_multiplier.max(1);
            let outside_penalty =
                area_out_plot * value * config.outside_penalty_multiplier * penalty_mult;
            let overlap_penalty =
                overlap_area * value * config.overlap_penalty_multiplier * penalty_mult;

            fitness += (base_fitness * area + tiebreak - outside_penalty - overlap_penalty) / area;
        }

        chromosome.fitness = Some(fitness);
        fitness
    }

    pub fn random_legal_position(&self, rng: &mut StdRng, w: u8, h: u8) -> (u8, u8) {
        let mut candidates = Vec::new();
        for y in 0..=self.height as i32 - h as i32 {
            for x in 0..=self.width as i32 - w as i32 {
                if self.is_legal(x, y, w as i32, h as i32) {
                    candidates.push((x as u8, y as u8));
                }
            }
        }
        if candidates.is_empty() {
            return (0, 0);
        }
        candidates[rng.random_range(0..candidates.len())]
    }

    fn is_legal(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        for i in 0..w {
            for j in 0..h {
                let cx = x + i;
                let cy = y + j;
                if cx < 0 || cy < 0 || cx >= self.width as i32 || cy >= self.height as i32 {
                    return false;
                }
                let sx = cx / self.subplot_size as i32;
                let sy = cy / self.subplot_size as i32;
                let idx = (sy * self.subplot_grid.0 as i32 + sx) as usize;
                if self.multipliers[idx] == 0 {
                    return false;
                }
            }
        }
        true
    }

    pub fn fits_cleanly(&self, x: u8, y: u8, w: u8, h: u8, occupied: &[bool]) -> bool {
        let (x, y, w, h) = (x as i32, y as i32, w as i32, h as i32);
        if x < 0 || y < 0 || x + w > self.width as i32 || y + h > self.height as i32 {
            return false;
        }
        for i in 0..w {
            for j in 0..h {
                let cx = x + i;
                let cy = y + j;
                let sx = cx / self.subplot_size as i32;
                let sy = cy / self.subplot_size as i32;
                let si = (sy * self.subplot_grid.0 as i32 + sx) as usize;
                if self.multipliers[si] == 0 {
                    return false;
                }
                if occupied[(cy * self.width as i32 + cx) as usize] {
                    return false;
                }
            }
        }
        true
    }

    pub fn mark_occupied(&self, x: u8, y: u8, w: u8, h: u8, occupied: &mut [bool]) {
        for i in 0..w as i32 {
            for j in 0..h as i32 {
                let idx = ((y as i32 + j) * self.width as i32 + (x as i32 + i)) as usize;
                occupied[idx] = true;
            }
        }
    }

    pub fn find_legal_position(
        &self,
        rng: &mut StdRng,
        w: u8,
        h: u8,
        occupied: &[bool],
        tries: usize,
    ) -> Option<(u8, u8)> {
        for _ in 0..tries {
            let x = rng.random_range(0..=self.width - w);
            let y = rng.random_range(0..=self.height - h);
            if self.fits_cleanly(x, y, w, h, occupied) {
                return Some((x, y));
            }
        }
        None
    }

    pub fn chromosome_is_legal(&self, chromosome: &Chromosome) -> bool {
        let mut occupied = vec![false; (self.width as usize) * (self.height as usize)];

        for gene in chromosome.genes.iter() {
            if !gene.placed {
                continue;
            }

            let (w, h) = if gene.rotated {
                (gene.h, gene.w)
            } else {
                (gene.w, gene.h)
            };

            if !self.fits_cleanly(gene.x, gene.y, w, h, &occupied) {
                return false;
            }
            self.mark_occupied(gene.x, gene.y, w, h, &mut occupied);
        }
        true
    }
}
