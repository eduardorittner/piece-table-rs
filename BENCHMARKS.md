# Benchmark Results

## Insert Character

Note that the only characters that have been tested so far are one-byte ascii characters, it may be that the performance profile of inserting multi-byte utf8 characters is different, but it shouldn't change that much. This hasn't been done yet because insertions should always happen at char boundaries, and guaranteeing that at runtime with random data involves at least *some* logic, which could influence the benchmarks. It probably can be done, it just isn't a priority.

### Random Insert
- Rope: ~250ns
- String: ~21μs (84x slower than Rope)
- PieceTable: ~165μs (660x slower than Rope)

### Start Insert
- Rope: ~130ns
- String: ~50μs (385x slower)
- PieceTable: ~26ns (5x faster than Rope)

### Middle Insert
- Rope: ~160ns
- String: ~20μs (125x slower)
- PieceTable: ~140μs (875x slower)

### End Insert
- Rope: ~170ns
- String: ~1.2ns (fastest)
- PieceTable: ~95μs (560x slower than Rope)

## Insert Small Text ("a")

### Random Insert
- Rope: ~240ns
- String: ~22μs (92x slower)
- PieceTable: ~160μs (670x slower)

### Start Insert
- Rope: ~130ns
- String: ~49μs (380x slower)
- PieceTable: ~29ns (4.5x faster than Rope)

### Middle Insert
- Rope: ~160ns
- String: ~18μs (115x slower)
- PieceTable: ~140μs (875x slower)

### End Insert
- Rope: ~170ns
- String: ~1.3ns (fastest)
- PieceTable: ~95μs (560x slower)

## Insert Medium Text ("This is some text.")

### Random Insert
- Rope: ~310ns
- String: ~22μs (71x slower)
- PieceTable: ~150μs (480x slower)

### Start Insert
- Rope: ~180ns
- String: ~50μs (280x slower)
- PieceTable: ~87ns (2x faster than Rope)

### Middle Insert
- Rope: ~230ns
- String: ~20μs (87x slower)
- PieceTable: ~140μs (610x slower)

### End Insert
- Rope: ~200ns
- String: ~10ns (fastest)
- PieceTable: ~90μs (450x slower)

## Insert Large Text (file contents)

### Random Insert
- Rope: ~2.5μs
- String: ~28μs (11x slower)
- PieceTable: ~190μs (76x slower)

### Start Insert
- Rope: ~1.6μs
- String: ~58μs (36x slower)
- PieceTable: ~2.2μs (1.4x slower)

### Middle Insert
- Rope: ~1.9μs
- String: ~25μs (13x slower)
- PieceTable: ~150μs (80x slower)

### End Insert
- Rope: ~2.2μs
- String: ~0.7μs (fastest)
- PieceTable: ~190μs (86x slower)

## Remove Character

### Random Remove
- Rope: ~242ns
- String: ~21μs (87x slower than Rope)
- PieceTable: ~292μs (1,206x slower than Rope)

### Start Remove
- Rope: ~164ns
- String: ~42μs (256x slower)
- PieceTable: ~195μs (1,189x slower)

### Middle Remove
- Rope: ~191ns
- String: ~17μs (89x slower)
- PieceTable: ~238μs (1,246x slower)

### End Remove
- Rope: ~197ns
- String: ~6ns (fastest)
- PieceTable: ~23ns (8.6x faster than Rope)

## Remove Small Text ("a")

### Random Remove
- Rope: ~183ns
- String: ~265μs (1,448x slower)
- PieceTable: ~251μs (1,372x slower)

### Start Remove
- Rope: ~159ns
- String: ~632μs (3,975x slower)
- PieceTable: ~203μs (1,277x slower)

### Middle Remove
- Rope: ~184ns
- String: ~238μs (1,293x slower)
- PieceTable: ~185μs (similar to Rope)

### End Remove
- Rope: ~167ns
- String: ~572ps (fastest)
- PieceTable: ~23ns (7.3x faster than Rope)

## Remove Medium Text ("This is some text.")

### Random Remove
- Rope: ~183ns
- String: ~265μs (1,448x slower)
- PieceTable: ~251μs (1,372x slower)

### Start Remove
- Rope: ~159ns
- String: ~632μs (3,975x slower)
- PieceTable: ~203μs (1,277x slower)

### Middle Remove
- Rope: ~184ns
- String: ~238μs (1,293x slower)
- PieceTable: ~185μs (similar to Rope)

### End Remove
- Rope: ~167ns
- String: ~572ps (fastest)
- PieceTable: ~23ns (7.3x faster than Rope)

## Remove Large Text (file contents)

### Random Remove
- Rope: ~2.1μs
- String: ~42ms (20,000x slower)
- PieceTable: ~1.2ms (571x slower)

### Start Remove
- Rope: ~1.6μs
- String: ~273ms (170,625x slower)
- PieceTable: ~858μs (536x slower)

### Middle Remove
- Rope: ~2.0μs
- String: ~16ms (8,000x slower)
- PieceTable: ~903μs (452x slower)

### End Remove
- Rope: ~1.6μs
- String: ~1.6μs (similar to Rope)
- PieceTable: ~219μs (200x slower)

## Remove Initial After Clone

- Rope: ~903ns
- String: ~22μs (24x slower)
- PieceTable: ~179μs (198x slower)

## Query Operations

### Slice Operations

#### Random Slice
- Rope: ~811ns
- PieceTable: ~46ns (17.6x faster than Rope)

#### Small Slice (64 chars)
- Rope: ~254ns
- PieceTable: ~38ns (6.7x faster than Rope)

#### Slice from Small Text
- Rope: ~409ns
- PieceTable: ~45ns (9.1x faster than Rope)

#### Whole Rope Slice
- Rope: ~33ns
- PieceTable: ~30ns (similar to Rope)

#### Whole Slice from Slice
- Rope: ~44ps
- PieceTable: ~28ns (636x slower than Rope)

### Length Operations

#### Character Count
- Rope: ~6.6ns
- String: ~86μs (13,030x slower than Rope)
- PieceTable: ~91μs (13,788x slower than Rope)

#### Byte Count
- Rope: ~6.8ns
- String: ~46ps (148x faster than Rope)
- PieceTable: ~46ps (150x slower than Rope)

#### Line Count
- Rope: ~6.6ns
- String: ~291μs (44,091x slower than Rope)
- PieceTable: Not directly supported (would require full string conversion)

## Conclusion

### Insert Operations
1. Rope is consistently fast for most insertion operations (except end inserts). This makes sense given its balanced tree structure.
2. String is fastest for end insertions but very slow for other operations. This makes sense since in strings, inserting at the end requires at most a reallocation, and at best a byte write and updating the length which is basically instant.
3. PieceTable shows the most varied performance:
   - Fastest for start inserts (beating both Rope and String)
   - Slowest for middle/random inserts
   - Competitive for end inserts
The start insert performance comes from direct access to the first node and VecDeque's circular nature, while middle inserts suffer from node traversal and splitting overhead.

### Remove Operations
1. Rope maintains consistent performance across all remove operations, showing its balanced structure works well for deletions too.
2. String shows extreme performance variations:
   - Fastest for end removes (often just updating length)
   - Extremely slow for start/middle removes (requiring memory shifts)
   - Particularly bad for large text removals in the middle/start
3. PieceTable shows interesting patterns:
   - Excellent performance for end removes (better than Rope)
   - Competitive performance for middle removes with small/medium text
   - Generally slower than Rope for random/start removes
   - Very slow for character removals

### Query operations

1. Rope's slicing depends on the text size, while piece table seems to have a somewhat stable performance regardless of text size.
2. PieceTable's byte length is as fast as string's length, which is just reading a usize and is basically instant.
