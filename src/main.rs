use ::rand::{RngExt, SeedableRng, rngs::StdRng};
use macroquad::prelude::*;
use std::{
    collections::HashMap,
    iter,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Copy)]
struct GeneratorData {
    id: u8,
    base_value: u32,
    width: u8,
    height: u8,
    gold: bool,
}

impl GeneratorData {
    fn new(id: u8, base_value: u32, width: u8, height: u8, gold: bool) -> Self {
        Self {
            id,
            base_value,
            width,
            height,
            gold,
        }
    }
}

#[derive(Clone)]
struct Gene {
    id: u8,
    x: u8,
    y: u8,
    rotated: bool,
    placed: bool,
}

impl From<GeneratorData> for Gene {
    fn from(value: GeneratorData) -> Self {
        Self {
            id: value.id,
            x: 0,
            y: 0,
            rotated: false,
            placed: false,
        }
    }
}
#[derive(Clone)]
struct Chromosome {
    genes: Vec<Gene>,
    fitness: i32,
}

impl Chromosome {
    fn new(gens: &Vec<(GeneratorData, u8)>) -> Self {
        let mut genes = Vec::new();

        for (generator, count) in gens {
            genes.extend(iter::repeat(Gene::from(*generator)).take(*count as usize));
        }

        Self { genes, fitness: 0 }
    }

    fn mutate(&mut self, rng: &mut StdRng, plot: &Plot) {
        let w = plot.width;
        let h = plot.height;
        for genes in self.genes.iter_mut() {
            if rng.random_bool(0.5) {
                genes.placed = !genes.placed;
                if genes.placed && rng.random_bool(0.5) {
                    genes.x = rng.random_range(0..w);
                    genes.y = rng.random_range(0..h);
                }
            }
        }
    }

    fn crossover(&mut self) {}
}

struct Population {
    chromosomes: Vec<Chromosome>,
    plot: Plot,
    gens: Vec<(GeneratorData, u8)>,
    rng: StdRng,
}

impl Population {
    fn new(pop_size: usize, gens: &Vec<(GeneratorData, u8)>, plot: &Plot, rng: StdRng) -> Self {
        let mut chromosomes = Vec::new();

        for i in 0..pop_size {
            chromosomes.push(Chromosome::new(gens));
        }

        Self {
            chromosomes,
            plot: plot.clone(),
            gens: gens.clone(),
            rng,
        }
    }

    fn mutate(&mut self) {
        for chromosome in self.chromosomes.iter_mut() {
            chromosome.mutate(&mut self.rng, &self.plot);
        }
    }

    fn crossover(&mut self) {
        for chromosome in self.chromosomes.iter_mut() {
            chromosome.crossover();
        }
    }

    fn evaluate(&mut self) {
        for chromosome in self.chromosomes.iter_mut() {
            self.plot.evaluate_chromosome(chromosome, &self.gens);
        }

        self.chromosomes.sort_unstable_by_key(|c| -c.fitness);
    }
}

#[derive(Clone)]
struct Plot {
    subplot_size: u8,
    subplot_grid: (u8, u8),
    multipliers: Vec<u8>,

    width: u8,
    height: u8,
}

impl Plot {
    fn new(subplot_size: u8, subplot_grid: (u8, u8), multipliers: Vec<u8>) -> Self {
        Self {
            subplot_size,
            subplot_grid,
            multipliers,

            width: subplot_size * subplot_grid.0,
            height: subplot_size * subplot_grid.1,
        }
    }

    fn evaluate_chromosome(
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
            let overlap_penalty = overlap_area * value;
            fitness += (base_fitness - outside_penalty - overlap_penalty) / area;
        }
        chromosome.fitness = fitness;
        fitness
    }
}

#[macroquad::main("Population Bounds")]
async fn main() {
    let gen_1 = GeneratorData::new(1, 1000, 2, 2, false);
    let gen_2 = GeneratorData::new(2, 2000, 2, 3, false);

    let mut gens: Vec<(GeneratorData, u8)> = Vec::new();
    gens.push((gen_1, 10));
    gens.push((gen_2, 5));

    let plot = Plot::new(7, (3, 4), vec![0, 0, 4, 3, 3, 3, 2, 2, 2, 1, 1, 1]);

    let rng = StdRng::seed_from_u64(31415926535);

    let population = Population::new(10, &gens, &plot, rng);

    let shared_pop = Arc::new(Mutex::new(population));
    let thread_pop = shared_pop.clone();

    thread::spawn(move || {
        for _ in 0..100 {
            let mut pop = thread_pop.lock().unwrap();
            pop.mutate();
            pop.crossover();
            pop.evaluate();
            drop(pop);
        }
    });

    render_population(shared_pop, &gens, plot.width, plot.height).await;
}
async fn render_population(
    shared_pop: Arc<Mutex<Population>>,
    gens: &[(GeneratorData, u8)],
    plot_width: u8,
    plot_height: u8,
) {
    let gen_map: HashMap<u8, GeneratorData> = gens.iter().map(|(g, _)| (g.id, *g)).collect();
    let scale = 17.0;

    let mut current_index = 0;

    loop {
        clear_background(GRAY);

        let pop = shared_pop.lock().unwrap();
        let pop_size = pop.chromosomes.len();

        let subplot_size = pop.plot.subplot_size as f32;
        let grid_w = pop.plot.subplot_grid.0 as usize;
        let grid_h = pop.plot.subplot_grid.1 as usize;

        for grid_y in 0..grid_h {
            for grid_x in 0..grid_w {
                let idx = grid_y * grid_w + grid_x;
                let multiplier = pop.plot.multipliers[idx];

                let mut bg_color = match multiplier {
                    4 => RED,
                    3 => YELLOW,
                    2 => LIME,
                    1 => GREEN,
                    0 => GRAY,
                    _ => WHITE,
                };
                bg_color.a = 0.5;

                let px = grid_x as f32 * subplot_size * scale;
                let py = grid_y as f32 * subplot_size * scale;
                let p_size = subplot_size * scale;

                draw_rectangle(px, py, p_size, p_size, bg_color);
                draw_rectangle_lines(px, py, p_size, p_size, 1.0, DARKGRAY);
            }
        }

        if is_key_pressed(KeyCode::Right) && current_index < pop_size - 1 {
            current_index += 1;
        }
        if is_key_pressed(KeyCode::Left) && current_index > 0 {
            current_index -= 1;
        }

        let chromosome = &pop.chromosomes[current_index];

        draw_rectangle_lines(
            0.0,
            0.0,
            plot_width as f32 * scale,
            plot_height as f32 * scale,
            4.0,
            BLACK,
        );

        for chromo in &chromosome.genes {
            if !chromo.placed {
                continue;
            }

            let generator = gen_map.get(&chromo.id).unwrap();

            let (w, h) = if chromo.rotated {
                (generator.height as f32, generator.width as f32)
            } else {
                (generator.width as f32, generator.height as f32)
            };

            let x = chromo.x as f32 * scale;
            let y = chromo.y as f32 * scale;

            let (box_color, text_color) = if generator.gold {
                (GOLD, BLACK)
            } else {
                (color_u8!(0, 0, 255, 50), WHITE)
            };

            draw_rectangle(x, y, w * scale, h * scale, box_color);
            draw_rectangle_lines(x, y, w * scale, h * scale, 2.0, BLACK);

            let text = chromo.id.to_string();
            let font_size = 30.0;
            let text_x = x + (w * scale * 0.5) - (font_size * 0.3);
            let text_y = y + (h * scale * 0.5) + (font_size * 0.3);

            draw_text(&text, text_x, text_y, font_size, text_color);
        }

        draw_text(
            &format!("Index: {}/{}", current_index, pop_size - 1),
            10.0,
            550.0,
            30.0,
            BLACK,
        );
        draw_text(
            &format!("Fitness: {}", chromosome.fitness),
            10.0,
            580.0,
            30.0,
            BLACK,
        );

        drop(pop);

        if is_key_pressed(KeyCode::S) {
            get_screen_data().export_png("population_render.png");
            println!("Saved image to population_render.png");
        }

        next_frame().await
    }
}
