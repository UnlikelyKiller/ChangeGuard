# Track U14 Spec: Semantic Indexing Concurrency Auto-Tuning

## Background
Parallel parse and embed routines during semantic indexing rely on `rayon`. If configured suboptimally, indexing tasks can degrade host responsiveness or hit socket limits on external servers.

## Objective
Dynamically adjust parallel indexing thread count parameters on startup matching available CPU topologies and request-rate budgets.

## Proposed Design
* Query system metrics (e.g. `num_cpus`) to scale active concurrency ranges.
* Cap active threadpools during remote embedding tasks to prevent network timeouts.
