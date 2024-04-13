use rand::Rng;
use rayon::prelude::*;
use std::sync::atomic::{AtomicI32, Ordering};

const TARGET: &str = "Hello, World! This is a genetic algorithm in Rust!";
const POPULATION_SIZE: usize = 1000;
const MUTATION_RATE: f64 = 0.01;
const GENERATIONS: u32 = 1000000;
const INFINITE_GENERATIONS: bool = true;
const SELECT_FROM_TOP_PERCENT: f64 = 0.1;

#[derive(Clone)]
struct Individual {
    genes: Vec<u8>,
    fitness: i32,
    calculated: bool,
}

impl Individual {
    fn new() -> Individual {
        let mut rng = rand::thread_rng();
        Individual {
            genes: (0..TARGET.len()).map(|_| rng.gen::<u8>()).collect(),
            fitness: 0,
            calculated: false,
        }
    }

    fn calculate_fitness(&mut self) -> i32 {
        if self.calculated {
            return self.fitness;
        }

        self.fitness = self
            .genes
            .iter()
            .zip(TARGET.as_bytes())
            // Count the number of genes that match the target
            .fold(0, |acc, (&g, &t)| acc + if g == t { 1 } else { 0 });
        self.calculated = true;

        self.fitness
    }

    fn genes_as_string(&self) -> String {
        self.genes.iter().map(|&g| g as char).collect()
    }
}

fn select(population: &[Individual]) -> &Individual {
    let top_idx = ((population.len() as f64) * (SELECT_FROM_TOP_PERCENT / 100.0)).ceil() as usize;
    let top = &population[..top_idx.max(1)];

    top.iter()
        .max_by(|x, y| x.fitness.cmp(&y.fitness))
        .expect("Non-empty population required")
}

fn crossover(parent1: &Individual, parent2: &Individual) -> Individual {
    let mut rng = rand::thread_rng();
    let crossover_point = rng.gen_range(0..TARGET.len());
    let child_genes: Vec<u8> = parent1
        .genes
        .iter()
        .take(crossover_point)
        .chain(parent2.genes.iter().skip(crossover_point))
        .copied()
        .collect();

    Individual {
        genes: child_genes,
        fitness: 0,
        calculated: false,
    }
}

fn mutate(individual: &mut Individual) {
    for gene in individual.genes.iter_mut() {
        if rand::thread_rng().gen::<f64>() < MUTATION_RATE {
            *gene = rand::thread_rng().gen::<u8>();
            individual.calculated = false;
        }
    }
}

fn main() {
    println!("Target: \x1b[33m{}\x1b[0m", TARGET);
    println!("Population size: \x1b[33m{}\x1b[0m", POPULATION_SIZE);
    println!("Mutation rate: \x1b[33m{}\x1b[0m", MUTATION_RATE);
    println!("Generations: \x1b[33m{}\x1b[0m", GENERATIONS);
    println!(
        "Select from top: \x1b[33m{}%\x1b[0m",
        SELECT_FROM_TOP_PERCENT
    );

    let mut population: Vec<Individual> = (0..POPULATION_SIZE)
        .into_par_iter()
        .map(|_| Individual::new())
        .collect();
    let max_fitness = AtomicI32::new(0);

    for generation in 0.. {
        if !INFINITE_GENERATIONS && generation >= GENERATIONS {
            break;
        }

        population.par_iter_mut().for_each(|individual| {
            let fitness = individual.calculate_fitness();
            max_fitness.fetch_max(fitness, Ordering::SeqCst);
            if fitness as usize == TARGET.len() {
                println!(
                    "\x1b[35mSolution found in generation {}!\x1b[0m, Genes: \x1b[33m{}\x1b[0m",
                    generation,
                    individual.genes_as_string()
                );
                std::process::exit(0);
            }
        });

        population.par_sort_by(|a, b| b.fitness.cmp(&a.fitness));

        let new_population: Vec<Individual> = (0..POPULATION_SIZE)
            .into_par_iter()
            .map(|_| {
                let parent1 = select(&population);
                let parent2 = select(&population);
                let mut child = crossover(parent1, parent2);
                mutate(&mut child);
                child
            })
            .collect();

        population = new_population;
    }

    println!("\x1b[31mNo exact solution found.\x1b[0m");
}
