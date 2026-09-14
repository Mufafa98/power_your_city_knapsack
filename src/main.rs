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
struct Chromosome {
    id: u8,
    x: u8,
    y: u8,
    rotated: bool,
    placed: bool,
}

impl From<GeneratorData> for Chromosome {
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

struct Population {
    chromosomes: Vec<Chromosome>,
}

impl Population {
    fn new(gens: &Vec<(GeneratorData, u8)>) -> Self {
        let mut chromosomes = Vec::new();

        for (generator, count) in gens {
            chromosomes.extend(iter::repeat(Chromosome::from(*generator)).take(*count as usize));
        }

        Self { chromosomes }
    }
}

#[macroquad::main("Population Bounds")]
async fn main() {
    let gen_1 = GeneratorData::new(1, 1000, 2, 2, false);
    let gen_2 = GeneratorData::new(2, 2000, 2, 3, false);

    let mut gens: Vec<(GeneratorData, u8)> = Vec::new();
    gens.push((gen_1, 10));
    gens.push((gen_2, 5));

    let mut population = Population::new(&gens);

    population.chromosomes[0].placed = true;
    population.chromosomes[0].rotated = true;
    population.chromosomes[0].x = 1;
    population.chromosomes[0].y = 1;

    render_population(&population, &gens).await;
}

async fn render_population(population: &Population, gens: &[(GeneratorData, u8)]) {
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

        for chromo in &population.chromosomes {
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
            let color = if generator.gold { GOLD } else { BLUE };

            draw_rectangle(x, y, w * scale, h * scale, color);
            draw_rectangle_lines(x, y, w * scale, h * scale, 2.0, BLACK);
        }

        // Press 'S' to save the current frame as a PNG
        if is_key_pressed(KeyCode::S) {
            get_screen_data().export_png("population_render.png");
            println!("Saved image to population_render.png");
        }

        next_frame().await
    }
}
