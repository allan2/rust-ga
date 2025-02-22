use args::{RunModel, TargetProblem, Args};
use rand::rngs::ThreadRng;

use bitstring::{Bitstring, LinearCrossover, LinearMutation, count_ones, hiff};
use population::Population;
use generation::{Generation, WeightedSelector}; 
use individual::Individual;

pub mod args;
pub mod individual;
pub mod population;
pub mod generation;
pub mod bitstring;

pub fn do_main(args: Args) {
    let scorer = hiff;

    let binary_tournament = Population::<Bitstring>::make_tournament_selector(2);
    let decimal_tournament = Population::<Bitstring>::make_tournament_selector(10);
    let weighted_selectors: Vec<WeightedSelector<Bitstring>> 
        = vec![(&Population::best_score, 5),
               (&Population::random, 20),
               (&binary_tournament, 50),
               (&decimal_tournament, 25)];

    let population
        = Population::new_bitstring_population(
            args.population_size, 
            args.bit_length, 
            match args.target_problem {
                TargetProblem::CountOnes => count_ones,
                TargetProblem::Hiff => hiff
            });
    assert!(!population.is_empty());

    let make_child = move |rng: &mut ThreadRng, generation: &Generation<Bitstring>| {
        make_child(scorer, rng, generation)
    };

    let mut generation = Generation::new(
        population,
        &weighted_selectors,
        &make_child
    );

    assert!(!generation.population.is_empty());
    let best = generation.best_individual();
    println!("{}", best);
    println!("Pop size = {}", generation.population.size());
    println!("Bit length = {}", best.genome.len());

    (0..args.num_generations).for_each(|generation_number| {
        generation = match args.run_model {
            RunModel::Serial => generation.next(),
            RunModel::Parallel => generation.par_next()
        };
        let best = generation.best_individual();
        // TODO: Change 2 to be the smallest number of digits needed for
        //  args.num_generations-1.
        println!("Generation {:2} best is {}", generation_number, best);
    });
}

fn make_child(scorer: impl Fn(&[bool]) -> i64, rng: &mut ThreadRng, generation: &Generation<Bitstring>) -> Individual<Bitstring> {
    let first_parent = generation.get_parent(rng);
    let second_parent = generation.get_parent(rng);

    let genome
        = first_parent.genome
            .two_point_xo(&second_parent.genome, rng)
            .mutate_one_over_length(rng);
    let score = scorer(&genome);
    Individual { genome, score }
}