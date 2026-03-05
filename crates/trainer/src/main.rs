use ai_player::Brain;
use bevy::log::info;
use genetic_rs_extras::{pb::ProgressObserver, plot::FitnessPlotter};
use neat::*;
use plotters::prelude::{IntoDrawingArea, SVGBackend};
use std::{io::Write as _, path::PathBuf};

// TODO add a clap cli to configure these parameters
const NB_GENERATIONS: u64 = 1000;
const NB_GENOME_PER_GEN: usize = 100;
const MUTATION_RATE: f32 = 0.05;
const MUTATION_PASSES: usize = 3;

const OUTPUT_DIR: &str = "./sim";

struct BestAgentObserver {
    output: PathBuf,
}

impl FitnessObserver<Brain> for BestAgentObserver {
    fn observe(&mut self, fitnesses: &[(Brain, f32)]) {
        let best = &fitnesses[0].0;

        let mut file = std::fs::File::create(&self.output).expect("failed to create best agent file");
        let serialized = ron::ser::to_string_pretty(best, ron::ser::PrettyConfig::default()) // TODO maybe configure
            .expect("failed to serialize best agent");
        file.write_all(serialized.as_bytes())
            .expect("failed to write best agent to file");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(OUTPUT_DIR)?;

    let dir = PathBuf::from(OUTPUT_DIR);

    let observer = BestAgentObserver { output: dir.join("best.ron")}
        .layer(ProgressObserver::new_with_default_style(NB_GENERATIONS))
        .layer(FitnessPlotter::new());

    let mut sim = GeneticSim::new(
        Vec::par_gen_random(NB_GENOME_PER_GEN),
        FitnessEliminator::builder()
                .observer(observer)
                .fitness_fn(trainer::fitness)
                .build_or_panic(),
            CrossoverRepopulator::new(MUTATION_RATE, ReproductionSettings {
                mutation_passes: MUTATION_PASSES,
                ..Default::default()
            }),
    );

    info!("beginning simulation");

    // sim.perform_generations(NB_GENERATIONS as usize);
    for i in 0..NB_GENERATIONS {
        sim.next_generation();
        info!("Generation {}/{}", i + 1, NB_GENERATIONS);
    }

    sim.eliminator.observer.0.1.finish();

    let plot_path = dir.join("fitness_plot.svg");

    let backend = SVGBackend::new(&plot_path, (800, 600));
    let drawing_area = backend.into_drawing_area();
    sim.eliminator.observer.1.plot(&drawing_area)?;
    drawing_area.present()?;
    info!("Fitness plot saved to {}", plot_path.display());

    Ok(())
}
