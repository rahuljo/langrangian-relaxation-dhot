### About this repo
Code repository for the master's thesis submitted as part of MSc. Scientific Computing at TU Berlin.

This repository contains code that captures the logic of how the experiments were implemented. Since the actual code is proprietary, this code is not runnable and is just meant to give an overview of the implementation.

The main things that we show:

- `graph.rs` — the `decompose` function captures the logic of how the full problem was decomposed into smaller subproblems 
- `main.rs` — the `run_subgradient_method` function presents the implementation of the subgradient method

The rest of the code (building the MIP, computing objectives, updating multipliers and subgradients) are proprietary or modify existing proprietary code which belong to Zuse Institut Berlin (ZIB). Some of those functions appear as stubs with a short description of what they do.
