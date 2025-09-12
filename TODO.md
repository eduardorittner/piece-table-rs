# TODO

This file tracks the remaining tasks for the piece-table project.

## [ ] 1. Complete the PieceTable Implementation

The main goal is to have a robust, correct, and fully functional implementation of the `PieceTable` data structure as defined by the `EditableText` trait.

**Relevant Files:**
*   `src/lib.rs`: Contains the `PieceTable` struct and its implementation.
*   `src/interface.rs`: Defines the `EditableText` trait that our data structures adhere to.

### Subtasks

*   **[x] Pass all proptests without failing:**
    *   **Description:** The property-based tests are now passing after fixing a bug in the `delete` function. All three implementations (`PieceTable`, `Baseline`, and `LineBuffer`) are now being compared.
    *   **Context:** The failing test was `property_tests::compare_implementations` in `src/lib.rs`.
    *   **Action:** Done.

*   **[ ] Add more complete unit tests:**
    *   **Description:** While the property tests are great for finding edge cases, targeted unit tests for specific scenarios (e.g., deleting across multiple nodes, inserting at boundaries) are still valuable.
    *   **Action:** Expand the `tests` module in `src/lib.rs` with more unit tests covering complex cases for both `insert` and `delete`.

*   **[ ] Implement remaining `EditableText` methods:**
    *   **Description:** The `EditableText` trait defines several methods (`replace`, `undo`, `redo`, `clear_history`) that are currently just placeholders in the `PieceTable` implementation.
    *   **Action:** Implement the logic for these methods. I suggest we start with `replace`, as it can be built using the existing `insert` and `delete` operations.

*   **[ ] Clean up warnings:**
    *   **Description:** The current codebase has several compiler warnings (e.g., unused variables and imports) that should be resolved to improve code quality and prevent bugs.
    *   **Action:** Run `cargo check` and address each of the reported warnings.

## [ ] 2. Data Structures Comparison

The goal of this task is to benchmark our `PieceTable` against other common text-editing data structures to understand its performance characteristics.

**Relevant Files:**
*   `benches/editing_benchmark.rs`: The benchmark suite powered by `criterion`.
*   `workloads/`: Directory containing text files that define the sequence of operations for our benchmarks.

### Subtasks

*   **[x] Implement a new data structure for comparison:**
    *   **Description:** A `LineBuffer` implementation using `Vec<String>` has been added to serve as another point of comparison.
    *   **Action:** Done.

*   **[ ] Implement the interface with `ropey`:**
    *   **Description:** `ropey` is a popular, production-grade rope data structure in Rust. Implementing our `EditableText` interface for `ropey` will provide a strong, optimized baseline for performance comparison.
    *   **Action:** Add the `ropey` crate as a dependency and create a new module (e.g., `src/ropey_impl.rs`) that wraps it.

*   **[ ] Implement the interface with the already existent `piece-table-rs`:**
    *   **Description:** To see how our implementation stacks up against prior art, we should also wrap the existing `piece-table` crate from crates.io.
    *   **Action:** Add the `piece-table` crate as a dependency and create a wrapper module for it.

*   **[ ] (Optional) Implement the interface with a gap buffer:**
    *   **Description:** A gap buffer is another classic text-editing data structure. Implementing it would provide another interesting point of comparison.
    *   **Action:** Implement a gap buffer from scratch or find a suitable crate and wrap it in a new module.

*   **[ ] Research other possible data structures:**
    *   **Description:** Explore other data structures that could be used for text editing and could fit our `EditableText` interface, such as a piece tree (a variation of a piece table using a self-balancing binary search tree).
    *   **Action:** Research and document potential candidates.

*   **[ ] Neovim plugin that exports edits in the workload format:**
    *   **Description:** To generate realistic workloads, we can create a Neovim plugin to record editing sessions and export them to our simple `INSERT`/`DELETE` text format.
    *   **Context:** This would likely involve writing a Lua script that hooks into Neovim's buffer change events (`nvim_buf_attach`) and writing the corresponding operations to a file.

*   **[ ] Analyze and compare benchmark results:**
    *   **Description:** After implementing the other data structures, we will need to run the benchmarks with a variety of workloads and analyze the results to draw meaningful conclusions.
    *   **Action:** Create different workload files (e.g., mostly deletions, mixed operations, large files) and run the full benchmark suite. The results from `criterion` (in `target/criterion`) can then be compiled into a summary, possibly with graphs, in our project's `README.md`.
