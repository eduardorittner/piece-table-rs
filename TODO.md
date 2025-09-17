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

*   **[ ] Clean up warnings:**
    *   **Description:** The current codebase has several compiler warnings (e.g., unused variables and imports) that should be resolved to improve code quality and prevent bugs.
    *   **Action:** Run `cargo check` and address each of the reported warnings.

## [ ] 2. Data Structures Comparison

The goal of this task is to benchmark our `PieceTable` against other common text-editing data structures to understand its performance characteristics.

**Relevant Files:**
*   `benches/*`: Benchmark files.

### Subtasks

*   **[ ] Research other possible data structures:**
    *   **Description:** Explore other data structures that could be used for text editing and could fit our `EditableText` interface, such as a piece tree (a variation of a piece table using a self-balancing binary search tree).
    *   **Action:** Research and document potential candidates.

*   **[ ] Analyze and compare benchmark results:**
    *   **Description:** After implementing the other data structures, we will need to run the benchmarks with a variety of workloads and analyze the results to draw meaningful conclusions.

## [ ] 3. Expand Text Handling API

The goal is to implement a more comprehensive API for text handling operations.

### Subtasks

*   **[x] Implement comparison operators:**
    *   **Description:** Added support for comparison operations using Rust's standard traits (PartialOrd, PartialEq, Eq)
    *   **Action:** Implemented the traits in src/lib.rs for PieceTable
    *   **Context:** Includes tests for all comparison cases (==, !=, <, >, <=, >=)

*   **[x] Implement conversion traits:**
    *   **Description:** Added conversion support between PieceTable and String/&str using From/Into traits
    *   **Action:** Implemented From<String>, From<&str>, From<PieceTable> for String, and From<PieceTable> for &str in src/lib.rs
    *   **Context:** Includes comprehensive tests for all conversion cases

*   **[x] Implement PTableSlice:**
    *   **Description:** Create an immutable view into a piece table that can be used like a string slice
    *   **Action:** Implement PTableSlice struct with appropriate trait implementations in src/lib.rs
    *   **Context:** PTableSlice has been implemented and works correctly, however it's still not working as intended where having a PTableSlice shouldn't prevent a PieceTable from being mutated.

## [ ] 4. Implement Missing Query Operations for PieceTable

The goal is to implement additional query operations that are available in Rope but not yet supported by PieceTable, to enable comprehensive benchmarking.

**Relevant Files:**
*   `src/lib.rs`: Contains the `PieceTable` struct and its implementation.
*   `benches/queries.rs`: Benchmark file that was updated to only include supported operations.

### Subtasks

*   **[ ] Implement `byte()` function:**
    *   **Description:** Add a method to get the byte at a specific position in the PieceTable.
    *   **Action:** Implement `pub fn byte(&self, byte_index: usize) -> Option<u8>` in `PieceTable` struct.
    *   **Context:** This was removed from queries.rs benchmarks as it's not yet supported.

*   **[ ] Implement `char()` function:**
    *   **Description:** Add a method to get the character at a specific position in the PieceTable.
    *   **Action:** Implement `pub fn char(&self, char_index: usize) -> Option<char>` in `PieceTable` struct.
    *   **Context:** This was removed from queries.rs benchmarks as it's not yet supported.

*   **[ ] Implement `line()` function:**
    *   **Description:** Add a method to get the line at a specific position in the PieceTable.
    *   **Action:** Implement `pub fn line(&self, line_index: usize) -> Option<String>` in `PieceTable` struct.
    *   **Context:** This was removed from queries.rs benchmarks as it's not yet supported.

*   **[ ] Implement index conversion functions:**
    *   **Description:** Add methods to convert between different index types (byte, char, line).
    *   **Action:** Implement the following methods in `PieceTable` struct:
        - `pub fn byte_to_char(&self, byte_index: usize) -> Option<usize>`
        - `pub fn byte_to_line(&self, byte_index: usize) -> Option<usize>`
        - `pub fn char_to_byte(&self, char_index: usize) -> Option<usize>`
        - `pub fn char_to_line(&self, char_index: usize) -> Option<usize>`
        - `pub fn line_to_byte(&self, line_index: usize) -> Option<usize>`
        - `pub fn line_to_char(&self, line_index: usize) -> Option<usize>`
    *   **Context:** These were removed from queries.rs benchmarks as they are not yet supported.

*   **[ ] Add comprehensive length functions:**
    *   **Description:** Add methods to get length in different units (chars, bytes, lines).
    *   **Action:** Implement the following methods in `PieceTable` struct:
        - `pub fn len_chars(&self) -> usize`
        - `pub fn len_bytes(&self) -> usize`
        - `pub fn len_lines(&self) -> usize`
    *   **Context:** Currently only `len()` (bytes) is supported, but having separate methods would align better with Rope's API.
