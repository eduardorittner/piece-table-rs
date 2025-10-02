use criterion::{Criterion, black_box, criterion_group, criterion_main};
use piece_table::PieceTable;
use rand::random;

const LARGE_TEXT: &str = include_str!("large.txt");
const MEDIUM_TEXT: &str = include_str!("medium.txt");
const SMALL_TEXT: &str = include_str!("small.txt");

fn find_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_node");

    group.bench_function("find_node_small", |bench| {
        let ptable = PieceTable::new(SMALL_TEXT);
        let len = ptable.len();
        bench.iter(|| {
            let idx = random::<u64>() as usize % (len);
            black_box(ptable.find_node_bench(black_box(idx)));
        })
    });

    group.bench_function("find_node_medium", |bench| {
        let ptable = PieceTable::new(MEDIUM_TEXT);
        let len = ptable.len();
        bench.iter(|| {
            let idx = random::<u64>() as usize % (len);
            black_box(ptable.find_node_bench(black_box(idx)));
        })
    });

    group.bench_function("find_node_large", |bench| {
        let ptable = PieceTable::new(LARGE_TEXT);
        let len = ptable.len();
        bench.iter(|| {
            let idx = random::<u64>() as usize % (len);
            black_box(ptable.find_node_bench(black_box(idx)));
        })
    });
}

criterion_group!(benches, find_node);
criterion_main!(benches);
