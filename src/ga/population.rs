use std::collections::VecDeque;

use rand::{RngExt, rngs::StdRng};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::{
    Config, GeneratorData,
    ga::{chromosome::Chromosome, plot::Plot},
};

pub struct Population {
    pub chromosomes: Vec<Chromosome>,
    pop_size: usize,
    pub plot: Plot,
    rng: StdRng,
    config: Config,
    gens: Vec<(GeneratorData, u8)>,
}

impl Population {
    pub fn new(
        pop_size: usize,
        gens: &Vec<(GeneratorData, u8)>,
        plot: &Plot,
        rng: StdRng,
        config: Config,
    ) -> Self {
        let mut chromosomes = Vec::new();
        let mut rng = rng;

        for _ in 0..pop_size {
            chromosomes.push(Chromosome::random(gens, &mut rng, plot));
        }

        Self {
            chromosomes,
            pop_size,
            plot: plot.clone(),
            rng,
            config,
            gens: gens.clone(),
        }
    }

    pub fn mutate(&mut self) {
        let mut mutants = Vec::new();
        for chromosome in self.chromosomes.iter() {
            let mut mutant = chromosome.mutate(&mut self.rng, &self.plot, &self.config);
            mutant.repair(&self.plot, &mut self.rng);
            mutants.push(mutant);
        }
        self.chromosomes.extend_from_slice(&mutants);
    }

    pub fn crossover(&mut self) {
        let elite = self.chromosomes[0].clone();
        let mut offsprings = Vec::new();
        for (i, chromosome) in self.chromosomes.iter().enumerate() {
            let parent = if self.rng.random_bool(0.5) {
                &elite
            } else {
                let mut idx = self.rng.random_range(0..self.chromosomes.len());
                while idx == i && self.chromosomes.len() > 1 {
                    idx = self.rng.random_range(0..self.chromosomes.len());
                }
                &self.chromosomes[idx]
            };
            let mut offspring = chromosome.crossover(&mut self.rng, parent);
            offspring.repair(&self.plot, &mut self.rng);
            offsprings.push(offspring);
        }
        self.chromosomes.extend_from_slice(&offsprings);
    }

    pub fn evaluate(&mut self) {
        self.chromosomes.par_iter_mut().for_each(|c| {
            self.plot.evaluate_chromosome(c, &self.config);
        });
    }

    pub fn selection(&mut self) {
        self.chromosomes
            .sort_unstable_by_key(|c| -c.fitness.unwrap());
        let mut old_pop = VecDeque::from(self.chromosomes.clone());
        let mut new_population: Vec<Chromosome> = Vec::with_capacity(self.pop_size);

        let elites = (self.pop_size as f32 * self.config.elitism) as usize;

        for _ in 0..elites {
            let chromosome = old_pop.pop_front().unwrap();
            new_population.push(chromosome);
        }

        let tourn_indiv = (self.pop_size as f32 * self.config.tournament) as usize;

        for _ in 0..tourn_indiv {
            let mut idx_best = self.rng.random_range(0..old_pop.len());
            let mut best = &old_pop[idx_best];
            for _ in 1..self.config.tournament_k {
                let idx = self.rng.random_range(0..old_pop.len());
                let c = &old_pop[idx];
                if c.fitness > best.fitness {
                    best = c;
                    idx_best = idx;
                }
            }
            let best = best.clone();
            old_pop.swap_remove_back(idx_best);
            new_population.push(best);
        }

        while new_population.len() < self.pop_size {
            let chromosome = old_pop
                .swap_remove_back(self.rng.random_range(0..old_pop.len()))
                .unwrap();
            new_population.push(chromosome);
        }

        self.chromosomes = new_population;
    }

    pub fn polish_top(&mut self) {
        let top_n = self.chromosomes.len().min(self.config.polish_top_n);

        for ci in 0..top_n {
            let mut improved = true;
            while improved {
                improved = false;
                improved |= self.polish_nudges(ci);
                improved |= self.polish_swaps(ci, 80);
            }
        }

        self.chromosomes
            .sort_unstable_by_key(|c| -c.fitness.unwrap());
    }

    fn polish_nudges(&mut self, ci: usize) -> bool {
        let mut improved = false;

        for gi in 0..self.chromosomes[ci].genes.len() {
            let original = self.chromosomes[ci].genes[gi].clone();
            let current_fit = self.chromosomes[ci].fitness.unwrap();

            let mut best_gene = original.clone();
            let mut best_fit = current_fit;

            for (dx, dy, rot) in [
                (-1i32, 0i32, false),
                (1, 0, false),
                (0, -1, false),
                (0, 1, false),
                (0, 0, true),
            ] {
                let mut g = original.clone();
                g.x = ((g.x as i32 + dx).clamp(0, self.plot.width as i32 - 1)) as u8;
                g.y = ((g.y as i32 + dy).clamp(0, self.plot.height as i32 - 1)) as u8;
                if rot {
                    g.rotated = !g.rotated;
                }

                self.chromosomes[ci].genes[gi] = g;
                self.chromosomes[ci].fitness = None;
                let f = self
                    .plot
                    .evaluate_chromosome(&mut self.chromosomes[ci], &self.config);

                if f > best_fit {
                    best_fit = f;
                    best_gene = self.chromosomes[ci].genes[gi].clone();
                }
            }

            self.chromosomes[ci].genes[gi] = best_gene;
            if best_fit > current_fit {
                improved = true;
            }

            self.chromosomes[ci].fitness = None;
            self.plot
                .evaluate_chromosome(&mut self.chromosomes[ci], &self.config);
        }

        improved
    }

    fn polish_swaps(&mut self, ci: usize, tries: usize) -> bool {
        let mut improved = false;

        for _ in 0..tries {
            let n = self.chromosomes[ci].genes.len();
            if n < 2 {
                break;
            }

            let gi = self.rng.random_range(0..n);
            let mut gj = self.rng.random_range(0..n);
            while gj == gi {
                gj = self.rng.random_range(0..n);
            }

            let a = &self.chromosomes[ci].genes[gi];
            let b = &self.chromosomes[ci].genes[gj];

            if !a.placed || !b.placed || a.id == b.id {
                continue;
            }

            let current_fit = self.chromosomes[ci].fitness.unwrap();

            let mut trial = self.chromosomes[ci].clone();
            let (ax, ay, ar) = (a.x, a.y, a.rotated);
            let (bx, by, br) = (b.x, b.y, b.rotated);

            trial.genes[gi].x = bx;
            trial.genes[gi].y = by;

            trial.genes[gj].x = ax;
            trial.genes[gj].y = ay;

            let mut found_legal = false;
            for (rot_i, rot_j) in [(br, ar), (!br, ar), (br, !ar), (!br, !ar)] {
                trial.genes[gi].rotated = rot_i;
                trial.genes[gj].rotated = rot_j;

                if self.plot.chromosome_is_legal(&trial) {
                    found_legal = true;
                    break;
                }
            }

            if !found_legal {
                continue;
            }

            trial.fitness = None;
            let f = self.plot.evaluate_chromosome(&mut trial, &self.config);

            if f > current_fit {
                trial.fitness = Some(f);
                self.chromosomes[ci] = trial;
                improved = true;
            }
        }

        improved
    }

    pub fn inject_fresh(&mut self, fraction: f32) {
        let n = ((self.pop_size as f32 * fraction) as usize).max(1);
        let len = self.chromosomes.len();
        for i in len - n..len {
            let mut c = Chromosome::random(&self.gens, &mut self.rng, &self.plot);
            c.repair(&self.plot, &mut self.rng);
            self.chromosomes[i] = c;
        }
    }

    pub fn deduplicate(&mut self) {
        use std::collections::HashSet;

        let mut seen: HashSet<u64> = HashSet::with_capacity(self.chromosomes.len());
        let mut dups: Vec<usize> = Vec::new();

        for (i, c) in self.chromosomes.iter().enumerate() {
            let h = chromosome_hash(c);
            if !seen.insert(h) {
                dups.push(i);
            }
        }

        for i in dups {
            let mut m = self.chromosomes[i].mutate(&mut self.rng, &self.plot, &self.config);
            m.repair(&self.plot, &mut self.rng);
            m.fitness = None;
            self.plot.evaluate_chromosome(&mut m, &self.config);
            self.chromosomes[i] = m;
        }
    }
}
fn chromosome_hash(c: &Chromosome) -> u64 {
    // FNV-1a over the placed genes' (id, x, y, rotated). Order-independent
    // is nicer but the gene order is fixed, so a simple sequential hash works.
    let mut h: u64 = 0xcbf29ce484222325;
    for g in c.genes.iter() {
        if !g.placed {
            continue;
        }
        for byte in [g.id, g.x, g.y, g.rotated as u8] {
            h ^= byte as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
    }
    h
}
