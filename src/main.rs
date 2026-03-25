#![allow(unused)]

use grb::prelude::*;
use grb::parameter::{DoubleParam, IntParam};
use std::time::{Duration, Instant};
use clap::Parser;

mod graph;

use graph::Graph;


// CLI Arguments
#[derive(Parser, Debug)]
#[command(name = "dhot")]
#[command(about = "District heating Lagrangian relaxation solver", long_about = None)]
struct Args {
    /// Name of the instance JSON file (without extension) in data/graphs/
    #[arg(short = 'f', long = "file_name")]
    file_name: String,

    /// Solve the LP relaxation instead of the MILP
    #[arg(short = 'r', long = "lp_relaxation")]
    lp_relaxation: bool,

    /// Use Lagrangian relaxation (subgradient method); if false, solve out-of-box
    #[arg(short = 'l', long = "lagrange")]
    lagrange: bool,

    /// Partition size: number of half-month boundaries between partition endpoints.
    /// 0 = full decomposition (every time step is its own subproblem).
    /// See `decompose` in graph.rs for details.
    #[arg(short = 'p', long = "partition_size")]
    partition_size: u32,

    /// Wall-clock time limit for the entire subgradient run, in hours
    #[arg(short = 't', long = "time_limit_hours")]
    time_limit_hours: f64,

    /// Whether Gurobi should print solver output to the log (1 = yes, 0 = no)
    #[arg(short = 'o', long = "activate_output")]
    activate_output: i32,

    /// Gurobi relative MIP gap at which each subproblem is considered solved
    #[arg(short = 'g', long = "mip_gap")]
    mip_gap: f64,
}


// This function implements the Out-of-box Method
fn solve_out_of_box(
    graph: &mut Graph,
    instance_name: &str,
    lp_relaxation: bool,
    activate_output: i32,
    mip_gap: f64,
) {
    // Build the full model, set objective, and optimize
    unimplemented!("[not shown — uses proprietary DHOT base library]")
}

fn run_subgradient_method(
    graph: &mut Graph,
    partition_size: u32, 
    time_limit_hours: f64, 
    instance_name: &str,
    lp_relaxation: bool,
    activate_output: i32,
    mip_gap: f64,
) {
    
    // To construct the decomposed subproblems:
    // 1. we first divide the time steps into a parition of neighboring time steps
    // 2. then we treat each partition as a complete time horizon and construct a model
    //    on this time horizon  
    let (
        mut model_list,
        mut objective_list,
        time_idx_start_list,
        time_idx_end_list
    ) = graph.decompose(
        activate_output,
        partition_size,
        lp_relaxation,
        remove_all_time_linking_constraints,
    );


    // Name of the output file to store the objective and run time per iteration
    let output_file_name = generate_file_name(
        instance_name,
        lp_relaxation,
        partition_size,
        mip_gap,
    );
    let output_file_path = format!("./output/{output_file_name}.csv");

    // CSV writer for to store at every iteration: the current objective, 
    // best objective so far, and time taken.
    let mut obj_runtime_wtr = csv::Writer::from_path(&output_file_path).unwrap();
    obj_runtime_wtr.write_record([
        "Iteration", "Objective", "Runtime", "Best until now",
    ]).unwrap();

    let mut total_runtime = 0.0;
    let mut total_objective = 0.0;
    let mut best_until_now = f64::NEG_INFINITY;

    // Step-size parameters
    let alpha = 0.01;  
    let mut decay = 0.75;

    let mut step_size;
    let mut iteration: u32 = 0;

    // Initialise all Lagrange multipliers to 1
    println!("Initialising multipliers");
    graph.initialize_multipliers();
    
    // In our experiments, we set a time limit of 32 hours as stopping criterion
    let time_limit = Duration::from_secs_f64(time_limit_hours * 3600.0);
    let start = Instant::now();

    // Enter the while loop as long as 32 hours have not elapsed
    while start.elapsed() < time_limit {

        // Adapt step size based on the number of iterations
        if iteration == 250 { decay = 0.875; }
        if iteration == 500 { decay = 1.0;   }
        step_size = alpha / ((iteration + 1) as f64).powf(decay);
        println!("Iteration: {}  step size: {:.6}", iteration, step_size);

        // Initialize subgradients to 0
        graph.initialize_subgradients();

        let mut runtime_iteration = 0.0;

        // Iterate across all subproblems
        for (model_idx, submodel) in model_list.iter_mut().enumerate() {
            
            // Get the index of the time step at the start and end of the time horizon
            let time_idx_start = time_idx_start_list[model_idx];
            let time_idx_end   = time_idx_end_list[model_idx];
            println!("  Subproblem #{}: time steps {} – {}", model_idx, time_idx_start, time_idx_end);

            // Update the objective of the subproblem by adding the penalty term
            let mut objective = objective_list[model_idx].clone();
            Graph::add_lagrangian_penalty(
                graph,
                &mut objective,
                time_idx_start,
                time_idx_end,
                remove_all_time_linking_constraints,
            );
            submodel.set_objective(objective, Minimize);

            // set target MIP Gap
            submodel.set_param(DoubleParam::MIPGap, mip_gap).unwrap();
            // solve the subproblem
            submodel.optimize().unwrap();

            // Accumulate subgradients for each subproblem
            graph.update_subgradients(
                submodel,
                time_idx_start,
                time_idx_end,
                remove_all_time_linking_constraints,
            );

            // Add contribution to runtime and final objective
            runtime_iteration += submodel.get_attr(attr::Runtime).unwrap();
            total_objective    += submodel.get_attr(attr::ObjVal).unwrap();
        }

        // Update all multipliers using accumulated subgradients
        println!("Updating multipliers");
        graph.update_multipliers(step_size, graph.scenario_data.time_period);

        // Update the iteration count, runtime, and best objective so far
        iteration += 1;
        total_runtime += runtime_iteration;
        best_until_now = best_until_now.max(total_objective);
        println!("  Iteration objective:    {}", total_objective);
        println!("  Best dual bound so far: {}", best_until_now);
        println!("  Elapsed: {:.4} h", start.elapsed().as_secs_f64() / 3600.0);

        // Save current objective, total runtime, and best objective so far
        obj_runtime_wtr.write_record([
            iteration.to_string(),
            total_objective.to_string(),
            runtime_iteration.to_string(),
            best_until_now.to_string(),
        ]).unwrap();

        total_objective = 0.0;
    }

    // Save the data to CSV
    obj_runtime_wtr.flush().unwrap();
    println!("Total runtime:   {}", total_runtime);
    println!("Best dual bound: {}", best_until_now);
}

fn main() {
    println!("DHOT — Lagrangian Relaxation");

    let env  = Env::new("./output/logfile.log").unwrap();
    let args = Args::parse();

    let lp_relaxation     = args.lp_relaxation; // whether to solve LP or MILP
    let lagrange          = args.lagrange; // whether to run Lagrange or Out-of-box
    let partition_size = args.partition_size; // the partition size to choose if Lagrange
    let time_limit_hours  = args.time_limit_hours; // stopping criterion
    let activate_output   = args.activate_output; // whether to write Gurobi logs
    let mip_gap           = args.mip_gap; // target optimality gap

    // Load the district heating network from input JSON data instance.
    let file_path = format!("./data/graphs/{file_name}.json");
    let mut graph = Graph::new_from_file(&file_path, time_granularity);

    if lagrange {     
        // Solve using subgradient method    
        run_subgradient_method(
            &mut graph,
            partition_size,
            time_limit_hours, 
            instance_name,
            lp_relaxation,
            activate_output,
            mip_gap,
        );
    } else {
        // Solve out of box
        solve_out_of_box(
            &mut graph,
            lp_relaxation,
            activate_output,
            mip_gap,
        );
    }
}
