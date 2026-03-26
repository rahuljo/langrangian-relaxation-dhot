use grb::prelude::*;

// The Graph struct contains the network components of the network flow problem
pub struct Graph {
    pub scenario_data: ScenarioData, 
    pub nodes: Vec<Node>,            
    pub edges: Vec<Edge>,            
}

impl Graph {

    // Builds a Gurobi MILP for the time horizon idx_start to idx_end
    pub fn create_mip(&mut self, activate_output: i32, idx_start: u32, idx_end: u32) -> Model {
        unimplemented!("[modifies proprietary code]")
    }

    // Returns the linear cost objective for the time horizon
    pub fn get_objective(&mut self, id: u32, idx_start: u32, idx_end: u32) -> LinExpr {
        unimplemented!("[modifies proprietary code]")
    }

    // Adds the Lagrangian penalty terms to the subproblem objective
    pub fn add_lagrangian_penalty(
        graph: &mut Graph,
        objective: &mut LinExpr,
        idx_start: u32,
        idx_end: u32,
    ) {
        unimplemented!("[modifies proprietary code]")
    }

    // Initialises a multiplier vector where each entry corresponds to one
    // relaxed time-linking constraint. It sets an initial value of 1 for all entries.
    pub fn initialize_multipliers(&mut self) {
        unimplemented!("[modifies proprietary code]")
    }

    // Initialises a subgradient vector of the same length as the multiplier
    // vector. Sets all entries to 0.
    pub fn initialize_subgradients(&mut self) {
        unimplemented!("[modifies proprietary code]")
    }

    // Overwrites the subgradients at every iteration
    pub fn update_subgradients(
        &mut self,
        submodel: &mut Model,
        idx_start: u32,
        idx_end: u32,
        remove_all_time_linking_constraints: bool,
    ) {
        unimplemented!("[modifies proprietary code]")
    }

    // Updates each multiplier using accumulated subgradients after solving all subproblems
    pub fn update_multipliers(&mut self, step_size: f64, time_period: u32) {
        unimplemented!("[modifies proprietary code]")
    }

    // this function partitions the time steps and creates one subproblem per partition
    pub fn decompose(
        &mut self,
        activate_output: i32,
        partition_size: u32,
        lp_relaxation: bool,
    ) -> (Vec<Model>, Vec<LinExpr>, Vec<u32>, Vec<u32>) {
        let mut model_list = Vec::new();
        let mut objective_list = Vec::new();
        let mut time_idx_start_list = Vec::new();
        let mut time_idx_end_list = Vec::new();

        println!("time range max: {}", self.scenario_data.time_period);
        // Get a list of all time steps that will be a boundary of two partitions
        let mut timestamp_indices = get_timestamp_indices(
            partition size
        );
        // Append the final time step so the last partition is always closed
        timestamp_indices.push(self.scenario_data.time_period as usize);

        // Iterate through pairs of timestamp indices and create a subproblem
        // with this pair as the time horizon
        let mut idx_start = 0;
        for &t in timestamp_indices {
            let idx_end = std::cmp::min(t as u32, self.scenario_data.time_period);

            // Build the MIP submodel for this time horizon
            let mut model = self.create_mip(activate_output, idx_start, idx_end);

            // If lp_relaxation is enabled then relax all integer constraints
            if lp_relaxation {
                create_lp_relaxation(&mut model);
            }

            // Get the objective for this time horizon
            let objective = self.get_objective(0, idx_start, idx_end);
            // Accumulate the list of subproblems, objectives, and time indices
            model_list.push(model);
            objective_list.push(objective);
            time_idx_start_list.push(idx_start);
            time_idx_end_list.push(idx_end);

            idx_start = idx_end;
            if idx_start == self.scenario_data.time_period {
                break;
            }
        }

        // Handle the final partition if the horizon doesn't divide evenly
        if idx_start < self.scenario_data.time_period {
            let idx_end = self.scenario_data.time_period;
            let mut model = self.create_mip(activate_output, idx_start, idx_end);
            if lp_relaxation {
                create_lp_relaxation(&mut model);
            }
            let objective = self.get_objective(0, idx_start, idx_end);
            model_list.push(model);
            objective_list.push(objective);
            time_idx_start_list.push(idx_start);
            time_idx_end_list.push(idx_end);
        }

        (model_list, objective_list, time_idx_start_list, time_idx_end_list)
    }

}