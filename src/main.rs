use ::rand::{SeedableRng, rngs::StdRng};
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::ga::{Plot, Population};

mod ga;

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

struct Config {
    elitism: f32,
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
    let config = Config { elitism: 0.1 };

    let mut population = Population::new(10, &gens, &plot, rng, config);

    let gen_map: HashMap<u8, GeneratorData> = gens.iter().map(|(g, _)| (g.id, *g)).collect();
    let scale = 17.0;

    let mut current_index: usize = 0;
    let mut generation: u64 = 0;
    let mut paused = false;
    let mut gens_per_frame: usize = 1;

    loop {
        if !paused {
            for _ in 0..gens_per_frame {
                population.mutate();
                population.crossover();
                population.evaluate();
                population.selection();
                generation += 1;
            }
        }

        if is_key_pressed(KeyCode::Right) && current_index + 1 < population.chromosomes.len() {
            current_index += 1;
        }
        if is_key_pressed(KeyCode::Left) && current_index > 0 {
            current_index -= 1;
        }
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::Up) {
            gens_per_frame = (gens_per_frame * 2).min(4096);
        }
        if is_key_pressed(KeyCode::Down) {
            gens_per_frame = (gens_per_frame / 2).max(1);
        }

        clear_background(GRAY);

        let subplot_size = population.plot.subplot_size as f32;
        let grid_w = population.plot.subplot_grid.0 as usize;
        let grid_h = population.plot.subplot_grid.1 as usize;

        for grid_y in 0..grid_h {
            for grid_x in 0..grid_w {
                let idx = grid_y * grid_w + grid_x;
                let multiplier = population.plot.multipliers[idx];

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

        let chromosome = &population.chromosomes[current_index];

        draw_rectangle_lines(
            0.0,
            0.0,
            population.plot.width as f32 * scale,
            population.plot.height as f32 * scale,
            4.0,
            BLACK,
        );

        for gene in &chromosome.genes {
            if !gene.placed {
                continue;
            }

            let generator = gen_map.get(&gene.id).unwrap();

            let (w, h) = if gene.rotated {
                (generator.height as f32, generator.width as f32)
            } else {
                (generator.width as f32, generator.height as f32)
            };

            let x = gene.x as f32 * scale;
            let y = gene.y as f32 * scale;

            let (box_color, text_color) = if generator.gold {
                (GOLD, BLACK)
            } else {
                (color_u8!(0, 0, 255, 50), WHITE)
            };

            draw_rectangle(x, y, w * scale, h * scale, box_color);
            draw_rectangle_lines(x, y, w * scale, h * scale, 2.0, BLACK);

            let text = gene.id.to_string();
            let font_size = 30.0;
            draw_text(
                &text,
                x + (w * scale * 0.5) - (font_size * 0.3),
                y + (h * scale * 0.5) + (font_size * 0.3),
                font_size,
                text_color,
            );
        }

        draw_text(
            &format!(
                "Gen {}  index {}/{}  {}",
                generation,
                current_index,
                population.chromosomes.len() - 1,
                if paused { "PAUSED" } else { "running" },
            ),
            10.0,
            550.0,
            24.0,
            BLACK,
        );
        draw_text(
            &format!(
                "Fitness: {}   gens/frame: {}",
                chromosome.fitness, gens_per_frame
            ),
            10.0,
            580.0,
            24.0,
            BLACK,
        );

        if is_key_pressed(KeyCode::S) {
            get_screen_data().export_png("population_render.png");
            println!("Saved population_render.png");
        }

        next_frame().await;
    }
}
