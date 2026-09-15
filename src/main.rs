use ::rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use macroquad::prelude::{rand as _, *};

use crate::ga::{Plot, Population};

mod ga;

#[derive(Clone, Copy)]
struct GeneratorData {
    id: u8,
    base_value_: u32,
    width_: u8,
    height_: u8,
    gold_: bool,
}

impl GeneratorData {
    fn new(id: u8, base_value_: u32, width_: u8, height_: u8, gold_: bool) -> Self {
        Self {
            id,
            base_value_,
            width_,
            height_,
            gold_,
        }
    }
}

struct Config {
    elitism: f32,
    tournament: f32,
    tournament_k: usize,

    mutate: f64,
    mutate_pos: f64,
    mutate_rotation: f64,
    mutate_placement: f64,
    mutate_pos_by_nudge: f64,

    outside_penalty_multiplier: i32,
    overlap_penalty_multiplier: i32,

    polish_top_n: usize,
}

#[macroquad::main("Population Bounds")]
async fn main() {
    let blaze = GeneratorData::new(1, 125, 2, 3, false);
    let eco_tech = GeneratorData::new(2, 145, 2, 3, false);
    let eco_tech_g = GeneratorData::new(3, 145, 2, 3, true);
    let quantum = GeneratorData::new(4, 180, 2, 6, true);
    let sky = GeneratorData::new(5, 320, 3, 3, false);
    let sky_g = GeneratorData::new(6, 320, 3, 3, true);
    let eclipse = GeneratorData::new(7, 650, 6, 3, false);

    let mut gens: Vec<(GeneratorData, u8)> = Vec::new();
    gens.push((eco_tech, 38));
    gens.push((eco_tech_g, 2));
    gens.push((blaze, 20));
    gens.push((quantum, 3));
    gens.push((sky_g, 1));
    gens.push((sky, 1));
    gens.push((eclipse, 2));

    let plot = Plot::new(7, (3, 4), vec![0, 0, 4, 3, 3, 3, 2, 2, 2, 1, 1, 1]);

    let mut rng = StdRng::seed_from_u64(31415926535);
    let config = Config {
        elitism: 0.1,
        tournament: 0.8,
        tournament_k: 5,

        mutate: 0.11,
        mutate_pos: 0.8,
        mutate_rotation: 0.5,
        mutate_placement: 0.5,
        mutate_pos_by_nudge: 0.75,

        outside_penalty_multiplier: 3,
        overlap_penalty_multiplier: 7,

        polish_top_n: 1,
    };
    gens.shuffle(&mut rng);
    let mut population = Population::new(120, &gens, &plot, rng, config);

    let scale = 17.0;

    let mut current_index: usize = 0;
    let mut generation: u64 = 0;
    let mut paused = false;
    let mut gens_per_frame: usize = 64;

    loop {
        if !paused {
            for _ in 0..gens_per_frame {
                population.mutate();
                population.crossover();
                population.evaluate();
                population.selection();
                population.deduplicate();

                if generation % 50 == 0 {
                    population.polish_top();
                }
                if generation % 100 == 0 {
                    population.inject_fresh(0.25);
                }

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

            let (w, h) = if gene.rotated {
                (gene.h as f32, gene.w as f32)
            } else {
                (gene.w as f32, gene.h as f32)
            };

            let x = gene.x as f32 * scale;
            let y = gene.y as f32 * scale;

            let (mut box_color, text_color) = if gene.gold {
                (GOLD, BLACK)
            } else {
                (BLUE, WHITE)
            };

            box_color.a = 0.5;

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
                chromosome.fitness.unwrap(),
                gens_per_frame
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
