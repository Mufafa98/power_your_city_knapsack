use macroquad::prelude::*;
use std::{collections::HashMap, iter};

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
}

impl Chromosome {
    fn new(gens: &Vec<(GeneratorData, u8)>) -> Self {
        let mut genes = Vec::new();

        for (generator, count) in gens {
            genes.extend(iter::repeat(Gene::from(*generator)).take(*count as usize));
        }

        Self { genes }
    }

    fn mutate(&mut self) {
        let rng = rand::RandGenerator::new();
        for genes in self.genes.iter_mut() {
            if rng.gen_range(0.0, 1.0) < 0.5 {
                genes.placed = !genes.placed;
                if genes.placed && rng.gen_range(0.0, 1.0) < 0.5 {
                    genes.x = rng.gen_range(0, 10);
                    genes.y = rng.gen_range(0, 10);
                }
            }
        }
    }

    fn crossover(&mut self) {}
}

struct Population {
    chromosomes: Vec<Chromosome>,
}

impl Population {
    fn new(pop_size: usize, gens: &Vec<(GeneratorData, u8)>) -> Self {
        let mut chromosomes = Vec::new();

        for i in 0..pop_size {
            chromosomes.push(Chromosome::new(gens));
        }

        Self { chromosomes }
    }

    fn mutate(&mut self) {}

    fn crossover(&mut self) {}
}

#[macroquad::main("Population Bounds")]
async fn main() {
    let gen_1 = GeneratorData::new(1, 1000, 2, 2, false);
    let gen_2 = GeneratorData::new(2, 2000, 2, 3, false);

    let mut gens: Vec<(GeneratorData, u8)> = Vec::new();
    gens.push((gen_1, 10));
    gens.push((gen_2, 5));

    let mut population = Population::new(10, &gens);

    for i in 0..100 {
        population.mutate();
        population.crossover();
    }

    let mut chromosome = population.chromosomes[0].clone();
    chromosome.genes[0].placed = true;
    chromosome.genes[0].rotated = true;
    chromosome.genes[0].x = 10;
    chromosome.genes[0].y = 10;

    render_population(&chromosome, &gens).await;
}

async fn render_population(chromosome: &Chromosome, gens: &[(GeneratorData, u8)]) {
    let gen_map: HashMap<u8, GeneratorData> = gens.iter().map(|(g, _)| (g.id, *g)).collect();

    let scale = 50.0;
    let bound_width = 10.0;
    let bound_height = 10.0;

    loop {
        clear_background(GRAY);

        // Draw bounding box
        draw_rectangle_lines(
            0.0,
            0.0,
            bound_width * scale,
            bound_height * scale,
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
                (BLUE, WHITE)
            };

            draw_rectangle(x, y, w * scale, h * scale, box_color);
            draw_rectangle_lines(x, y, w * scale, h * scale, 2.0, BLACK);

            let text = chromo.id.to_string();
            let font_size = 30.0;
            // Offsets to roughly center the text
            let text_x = x + (w * scale * 0.5) - (font_size * 0.3);
            let text_y = y + (h * scale * 0.5) + (font_size * 0.3);

            draw_text(&text, text_x, text_y, font_size, text_color);
        }

        // Press 'S' to save the current frame as a PNG
        if is_key_pressed(KeyCode::S) {
            get_screen_data().export_png("population_render.png");
            println!("Saved image to population_render.png");
        }

        next_frame().await
    }
}
