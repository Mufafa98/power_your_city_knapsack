use std::collections::VecDeque;

use rand::{RngExt, rngs::StdRng};

use crate::{
    Config, GeneratorData,
    ga::{chromosome::Chromosome, plot::Plot},
};

pub struct Population {
    pub chromosomes: Vec<Chromosome>,
    pop_size: usize,
    pub plot: Plot,
    gens: Vec<(GeneratorData, u8)>,
    rng: StdRng,
    config: Config,
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

        for _ in 0..pop_size {
            chromosomes.push(Chromosome::new(gens));
        }

        Self {
            chromosomes,
            pop_size,
            plot: plot.clone(),
            gens: gens.clone(),
            rng,
            config,
        }
    }

    pub fn mutate(&mut self) {
        let mut mutants = Vec::new();
        for chromosome in self.chromosomes.iter() {
            let mutant = chromosome.mutate(&mut self.rng, &self.plot);
            mutants.push(mutant);
        }
        self.chromosomes.extend_from_slice(&mutants);
    }

    pub fn crossover(&mut self) {
        let mut offsprings = Vec::new();
        for chromosome in self.chromosomes.iter() {
            let parent = &self.chromosomes[self.rng.random_range(0..self.chromosomes.len())];
            let offspring = chromosome.crossover(&mut self.rng, &parent);
            offsprings.push(offspring);
        }
        self.chromosomes.extend_from_slice(&offsprings);
    }

    pub fn evaluate(&mut self) {
        for chromosome in self.chromosomes.iter_mut() {
            self.plot.evaluate_chromosome(chromosome, &self.gens);
        }
    }

    pub fn selection(&mut self) {
        self.chromosomes.sort_unstable_by_key(|c| -c.fitness);
        let mut old_pop = VecDeque::from(self.chromosomes.clone());
        let mut new_population: Vec<Chromosome> = Vec::with_capacity(self.pop_size);

        let elites = (self.pop_size as f32 * self.config.elitism) as usize;

        for _ in 0..elites {
            let chromosome = old_pop.pop_front().unwrap();
            new_population.push(chromosome);
        }

        while new_population.len() < self.pop_size {
            let chromosome = old_pop
                .remove(self.rng.random_range(0..old_pop.len()))
                .unwrap();
            new_population.push(chromosome);
        }

        self.chromosomes = new_population;
        self.chromosomes.sort_unstable_by_key(|c| -c.fitness);
    }
}
