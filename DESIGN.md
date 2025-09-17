# Piece Table Optimization Design Document

## Overview

This document outlines optimization strategies for the piece table implementation, specifically focusing on improving the performance of length calculation and final string generation operations. Currently, these operations require traversing all nodes and accumulating results, which creates significant performance bottlenecks as demonstrated in the benchmark results.

## Current Implementation Analysis

### How Length and Final String Operations Work Currently

#### Length Calculation (`len()` method)
```rust
pub fn len(&self) -> usize {
    self.to_string().len()
}
```

**Current Process:**
1. Calls `to_string()` which traverses all nodes
2. Concatenates all node content into a single String
3. Returns the length of the resulting string
4. **Time Complexity:** O(n) where n is total number of nodes
5. **Space Complexity:** O(m) where m is total text size (temporary allocation)

#### Final String Generation (`to_string()` method)
```rust
impl<'a> Display for PieceTable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.nodes {
            write!(
                f,
                "{}",
                match node.kind {
                    NodeKind::Original => unsafe {
                        self.original
                            .get_unchecked(node.range.start..node.range.end)
                    },
                    NodeKind::Added => unsafe {
                        self.added.get_unchecked(node.range.start..node.range.end)
                    },
                }
            )?;
        }
        Ok(())
    }
}
```

**Current Process:**
1. Iterates through all nodes in order
2. For each node, copies the referenced text slice to the formatter
3. Accumulates all text into the final output
4. **Time Complexity:** O(n) where n is total number of nodes
5. **Space Complexity:** O(m) where m is total text size

### Performance Impact Analysis

Based on benchmark results from `BENCHMARKS.md`:

#### Length Operations Performance
- **Rope:** ~6.8ns (cached values)
- **String:** ~46ps (direct field access)
- **PieceTable:** ~91μs (**1,338,235x slower** than Rope)

This extreme performance difference occurs because:
- Rope maintains cached length metadata
- String has direct length field access
- PieceTable rebuilds the entire string to calculate length

#### String Conversion Performance
While not directly benchmarked, `to_string()` performance is similarly poor because it requires the same node traversal and concatenation process as length calculation.

### Insert/Delete Operations Impact

When insert or delete operations occur:
1. New nodes are created or existing nodes are modified
2. Node sequence is updated
3. **No caching invalidation occurs** (since there's no cache)
4. Subsequent length/string operations pay full traversal cost

## Proposed Optimization Strategies

### Strategy 1: Cached Length with Incremental Updates

#### Design
Maintain a running total length that's updated incrementally during insert/delete operations.

```rust
pub struct PieceTable<'a> {
    original: &'a str,
    added: String,
    nodes: VecDeque<Node>,
    cached_length: usize, // New field
    length_valid: bool,   // New field
}
```

#### Implementation Details

**Length Calculation:**
```rust
pub fn len(&self) -> usize {
    if self.length_valid {
        self.cached_length
    } else {
        // Fallback to recalculation if cache is invalid
        self.recalculate_length()
    }
}

fn recalculate_length(&self) -> usize {
    self.nodes.iter().map(|n| n.range.len()).sum()
}
```

**Insert Operation Updates:**
```rust
pub fn insert(&mut self, data: &str, offset: usize) {
    let data_len = data.len();
    // ... existing insert logic ...
    
    // Update cached length
    if self.length_valid {
        self.cached_length += data_len;
    }
}
```

**Delete Operation Updates:**
```rust
pub fn delete(&mut self, range: Range<usize>) {
    let deleted_len = range.end - range.start;
    // ... existing delete logic ...
    
    // Update cached length
    if self.length_valid {
        self.cached_length -= deleted_len;
    }
}
```

**Cache Invalidation:**
```rust
fn invalidate_length_cache(&mut self) {
    self.length_valid = false;
}
```

#### Performance Analysis
- **Length Operation:** O(1) (cached lookup)
- **Insert/Delete:** O(1) additional overhead (simple arithmetic)
- **Memory Overhead:** 8-16 bytes per PieceTable instance
- **Expected Improvement:** ~1,000,000x faster length operations

#### Trade-offs
- **Pros:**
  - Massive performance improvement for length queries
  - Minimal memory overhead
  - Simple implementation
  - No impact on insert/delete performance

- **Cons:**
  - Requires careful cache invalidation logic
  - Adds complexity to insert/delete operations
  - Cache invalidation bugs could lead to incorrect length values

### Strategy 2: Cached Final String with Lazy Invalidation

#### Design
Maintain a cached copy of the final string that's invalidated on modifications and regenerated on demand.

```rust
pub struct PieceTable<'a> {
    original: &'a str,
    added: String,
    nodes: VecDeque<Node>,
    cached_string: Option<String>, // New field
    string_generation_threshold: usize, // Configurable threshold
}
```

#### Implementation Details

**String Generation:**
```rust
impl<'a> Display for PieceTable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(cached) = &self.cached_string {
            write!(f, "{}", cached)
        } else {
            // Generate string on the fly
            let mut result = String::new();
            for node in &self.nodes {
                match node.kind {
                    NodeKind::Original => unsafe {
                        result.push_str(
                            self.original.get_unchecked(node.range.start..node.range.end)
                        )
                    },
                    NodeKind::Added => unsafe {
                        result.push_str(
                            self.added.get_unchecked(node.range.start..node.range.end)
                        )
                    },
                }
            }
            write!(f, "{}", result)
        }
    }
}
```

**Cache Management:**
```rust
fn update_string_cache(&mut self) {
    if self.should_cache_string() {
        self.cached_string = Some(self.generate_string_uncached());
    } else {
        self.cached_string = None;
    }
}

fn should_cache_string(&self) -> bool {
    // Don't cache if string is too large
    if let Some(cached) = &self.cached_string {
        cached.len() > self.string_generation_threshold
    } else {
        self.len() <= self.string_generation_threshold
    }
}

fn invalidate_string_cache(&mut self) {
    self.cached_string = None;
}
```

**Insert/Delete Integration:**
```rust
pub fn insert(&mut self, data: &str, offset: usize) {
    self.invalidate_string_cache();
    // ... existing insert logic ...
    self.update_string_cache(); // Optionally regenerate immediately
}
```

#### Performance Analysis
- **String Operation:** O(1) if cached, O(n) if not cached
- **Insert/Delete:** O(1) for invalidation, O(n) if immediate regeneration
- **Memory Overhead:** O(m) where m is text size (when cached)
- **Expected Improvement:** Up to 1,000,000x faster for cached strings

#### Trade-offs
- **Pros:**
  - Massive performance improvement for repeated string operations
  - Configurable based on memory constraints
  - Transparent to users

- **Cons:**
  - High memory usage for large documents
  - Cache regeneration can be expensive
  - Memory pressure for very large strings

### Strategy 3: Hybrid Approach with Selective Caching

#### Design
Combine both length and string caching with intelligent policies based on document size and usage patterns.

```rust
pub struct PieceTable<'a> {
    original: &'a str,
    added: String,
    nodes: VecDeque<Node>,
    
    // Length caching
    cached_length: usize,
    length_valid: bool,
    
    // String caching
    cached_string: Option<String>,
    cache_config: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enable_length_cache: bool,
    pub enable_string_cache: bool,
    pub max_string_cache_size: usize,
    pub cache_on_modification: bool,
}
```

#### Implementation Details

**Intelligent Cache Management:**
```rust
impl<'a> PieceTable<'a> {
    pub fn with_cache_config(string: &'a str, config: CacheConfig) -> Self {
        let mut pt = Self::new(string);
        pt.cache_config = config;
        pt.initialize_caches();
        pt
    }
    
    fn initialize_caches(&mut self) {
        if self.cache_config.enable_length_cache {
            self.cached_length = self.recalculate_length();
            self.length_valid = true;
        }
        
        if self.cache_config.enable_string_cache {
            self.update_string_cache();
        }
    }
    
    fn on_modification(&mut self) {
        if self.cache_config.enable_length_cache {
            // Length cache will be updated incrementally
        }
        
        if self.cache_config.enable_string_cache {
            if self.cache_config.cache_on_modification {
                self.update_string_cache();
            } else {
                self.invalidate_string_cache();
            }
        }
    }
}
```

**Adaptive Cache Behavior:**
```rust
fn adaptive_cache_update(&mut self) {
    let current_size = self.len();
    
    // Disable string caching for very large documents
    if current_size > self.cache_config.max_string_cache_size {
        self.cached_string = None;
        self.cache_config.enable_string_cache = false;
    }
    
    // Enable length caching for all documents (low overhead)
    if !self.cache_config.enable_length_cache {
        self.cache_config.enable_length_cache = true;
        self.cached_length = self.recalculate_length();
        self.length_valid = true;
    }
}
```

#### Performance Analysis
- **Length Operation:** O(1) (always cached)
- **String Operation:** O(1) if cached and within size limits, O(n) otherwise
- **Memory Overhead:** Configurable, typically O(1) for length + O(m) for string (when enabled)
- **Expected Improvement:** 10x - 1,000,000x depending on configuration and usage

#### Trade-offs
- **Pros:**
  - Best of both approaches
  - Configurable based on use case
  - Adaptive behavior
  - Memory efficient for large documents

- **Cons:**
  - Increased implementation complexity
  - Configuration management required
  - More potential edge cases

### Strategy 4: Node-based Length Accumulation

#### Design
Store cumulative lengths at each node to enable O(log n) length queries and O(log n) node finding.

```rust
#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    range: Range<usize>,
    cumulative_length: usize, // New field: total length up to this node
}

pub struct PieceTable<'a> {
    original: &'a str,
    added: String,
    nodes: VecDeque<Node>,
    total_length: usize, // New field
}
```

#### Implementation Details

**Node Management with Cumulative Lengths:**
```rust
fn update_cumulative_lengths(&mut self, start_idx: usize) {
    let mut cumulative = if start_idx == 0 {
        0
    } else {
        self.nodes[start_idx - 1].cumulative_length
    };
    
    for i in start_idx..self.nodes.len() {
        cumulative += self.nodes[i].range.len();
        self.nodes[i].cumulative_length = cumulative;
    }
    
    self.total_length = cumulative;
}
```

**Length Calculation:**
```rust
pub fn len(&self) -> usize {
    self.total_length
}
```

**Optimized Node Finding:**
```rust
fn find_node(&self, offset: usize) -> Option<(usize, usize)> {
    // Binary search for O(log n) performance
    let mut left = 0;
    let mut right = self.nodes.len();
    
    while left < right {
        let mid = (left + right) / 2;
        let node = &self.nodes[mid];
        
        if offset < node.cumulative_length {
            right = mid;
        } else {
            left = mid + 1;
        }
    }
    
    if left < self.nodes.len() {
        let node_start = if left == 0 {
            0
        } else {
            self.nodes[left - 1].cumulative_length
        };
        Some((left, node_start))
    } else {
        None
    }
}
```

**Insert/Delete with Cumulative Updates:**
```rust
pub fn insert(&mut self, data: &str, offset: usize) {
    let data_len = data.len();
    // ... existing insert logic ...
    
    // Update cumulative lengths from insertion point
    if let Some((node_idx, _)) = self.find_node(offset) {
        self.update_cumulative_lengths(node_idx);
    }
}
```

#### Performance Analysis
- **Length Operation:** O(1) (direct field access)
- **Node Finding:** O(log n) (binary search instead of linear)
- **Insert/Delete:** O(n) for cumulative length updates
- **Memory Overhead:** 8 bytes per node
- **Expected Improvement:** 1,000,000x faster length, 10-100x faster node finding

#### Trade-offs
- **Pros:**
  - Constant time length operations
  - Faster node finding for large documents
  - No cache invalidation issues
  - Predictable performance

- **Cons:**
  - O(n) update cost for insert/delete operations
  - Increased memory per node
  - More complex node management
  - May not be worth it for small documents

## Implementation Recommendations

### Phase 1: Length Caching (High Priority, Low Risk)
1. **Implement Strategy 1** first due to its high benefit/cost ratio
2. Length operations show the worst performance (1.3Mx slower)
3. Minimal implementation complexity and memory overhead
4. No risk of memory issues with large documents

### Phase 2: Configurable String Caching (Medium Priority, Medium Risk)
1. **Implement Strategy 2** with configurable thresholds
2. Provides massive benefits for repeated string operations
3. Include size limits to prevent memory issues
4. Make it opt-in for memory-constrained environments

### Phase 3: Advanced Optimizations (Low Priority, High Complexity)
1. Consider **Strategy 4** if benchmarks show node finding is a bottleneck
2. Implement **Strategy 3** hybrid approach if both length and string caching prove valuable
3. Requires extensive testing and performance analysis

## Configuration Options

### Runtime Configuration
```rust
impl<'a> PieceTable<'a> {
    pub fn enable_length_caching(&mut self, enabled: bool) {
        self.cache_config.enable_length_cache = enabled;
        if enabled {
            self.cached_length = self.recalculate_length();
            self.length_valid = true;
        } else {
            self.length_valid = false;
        }
    }
    
    pub fn enable_string_caching(&mut self, enabled: bool) {
        self.cache_config.enable_string_cache = enabled;
        if enabled {
            self.update_string_cache();
        } else {
            self.cached_string = None;
        }
    }
    
    pub fn set_string_cache_size_limit(&mut self, max_size: usize) {
        self.cache_config.max_string_cache_size = max_size;
        self.adaptive_cache_update();
    }
}
```

### Compile-time Configuration
```rust
#[cfg(feature = "length-caching")]
pub struct PieceTable<'a> {
    cached_length: usize,
    length_valid: bool,
    // ... other fields
}

#[cfg(feature = "string-caching")]
pub struct PieceTable<'a> {
    cached_string: Option<String>,
    // ... other fields
}
```

## Expected Performance Improvements

### Quantitative Estimates

Based on current benchmark data and proposed optimizations:

#### Length Operations
- **Current:** ~91μs
- **With Caching:** ~6ns (same as Rope)
- **Improvement:** ~15,000x faster

#### String Operations
- **Current:** ~91μs (similar to length)
- **With Caching:** ~6ns (cached), ~91μs (uncached)
- **Improvement:** Up to 15,000x faster for cached operations

#### Insert/Delete Operations
- **Current:** ~95-292μs (depending on operation type)
- **With Caching:** ~95-300μs (minimal overhead)
- **Impact:** < 5% performance degradation

### Qualitative Benefits

1. **Competitive Performance:** PieceTable will match Rope performance for query operations
2. **Memory Efficiency:** Configurable caching prevents memory issues
3. **Predictable Performance:** Consistent O(1) or O(log n) operations
4. **Scalability:** Performance remains consistent as document size grows

## Risk Analysis and Mitigation

### Memory Usage Risks
**Risk:** String caching could consume excessive memory for large documents
**Mitigation:**
- Implement size-based thresholds
- Provide configuration options
- Default to conservative limits
- Monitor memory usage and adapt dynamically

### Cache Coherence Risks
**Risk:** Cache invalidation bugs could lead to incorrect results
**Mitigation:**
- Comprehensive test suite
- Debug assertions
- Optional validation mode
- Clear documentation of cache invalidation points

### Performance Regression Risks
**Risk:** Caching overhead could impact insert/delete performance
**Mitigation:**
- Benchmark before/after changes
- Profile hot paths
- Provide opt-out options
- Use compile-time feature flags

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod cache_tests {
    use super::*;
    
    #[test]
    fn test_length_caching() {
        let mut pt = PieceTable::new("hello");
        pt.enable_length_caching(true);
        
        // Initial length
        assert_eq!(pt.len(), 5);
        
        // After insert
        pt.insert(" world", 5);
        assert_eq!(pt.len(), 11);
        
        // After delete
        pt.delete(5..11);
        assert_eq!(pt.len(), 5);
    }
    
    #[test]
    fn test_string_caching() {
        let mut pt = PieceTable::new("hello");
        pt.enable_string_caching(true);
        
        // Initial string
        assert_eq!(pt.to_string(), "hello");
        
        // After modification
        pt.insert(" world", 5);
        assert_eq!(pt.to_string(), "hello world");
    }
    
    #[test]
    fn test_cache_invalidation() {
        let mut pt = PieceTable::new("hello");
        pt.enable_length_caching(true);
        pt.enable_string_caching(true);
        
        // Modify and verify cache invalidation
        pt.insert("x", 0);
        assert_eq!(pt.len(), 6);
        assert_eq!(pt.to_string(), "xhello");
    }
}
```

### Integration Tests
1. **Property Testing:** Extend existing property tests to include cache validation
2. **Performance Regression:** Benchmark all operations before/after optimization
3. **Memory Usage:** Test with various document sizes and cache configurations
4. **Concurrency:** Test cache behavior in multi-threaded scenarios (if applicable)

### Benchmark Tests
Extend existing benchmarks to measure:
- Length operation performance with caching enabled/disabled
- String operation performance with various cache configurations
- Memory usage with different document sizes
- Insert/delete performance impact

## Conclusion

The proposed caching optimizations will dramatically improve PieceTable performance for length and string operations, making it competitive with Rope while maintaining its strengths in insert/delete operations. The phased implementation approach allows for incremental delivery and risk management.

Key recommendations:
1. **Prioritize length caching** for maximum impact with minimal risk
2. **Make string caching configurable** to balance performance and memory usage
3. **Provide extensive configuration options** for different use cases
4. **Implement comprehensive testing** to ensure cache correctness
5. **Document trade-offs clearly** for users making configuration decisions

These optimizations will transform PieceTable from having the worst query performance to being competitive with the best alternatives, while maintaining its excellent insert/delete characteristics.
