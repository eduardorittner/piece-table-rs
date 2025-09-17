extern crate criterion;
extern crate rand;
extern crate ropey;

use criterion::{Criterion, criterion_group, criterion_main};
use piece_table::PieceTable;
use rand::random;
use ropey::Rope;

const TEXT: &str = include_str!("large.txt");
const SMALL_TEXT: &str = include_str!("small.txt");

//----

fn rope_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_slice");

    group.bench_function("slice", |bench| {
        let rope = Rope::from_str(TEXT);
        let len = rope.len_chars();
        bench.iter(|| {
            let mut start = random::<u64>() as usize % (len + 1);
            let mut end = random::<u64>() as usize % (len + 1);
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            rope.slice(start..end);
        })
    });

    group.bench_function("slice_small", |bench| {
        let rope = Rope::from_str(TEXT);
        let len = rope.len_chars();
        bench.iter(|| {
            let mut start = random::<u64>() as usize % (len + 1);
            if start > (len - 65) {
                start = len - 65;
            }
            let end = start + 64;
            rope.slice(start..end);
        })
    });

    group.bench_function("slice_from_small_rope", |bench| {
        let rope = Rope::from_str(SMALL_TEXT);
        let len = rope.len_chars();
        bench.iter(|| {
            let mut start = random::<u64>() as usize % (len + 1);
            let mut end = random::<u64>() as usize % (len + 1);
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            rope.slice(start..end);
        })
    });

    group.bench_function("slice_whole_rope", |bench| {
        let rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.slice(..);
        })
    });

    group.bench_function("slice_whole_slice", |bench| {
        let rope = Rope::from_str(TEXT);
        let len = rope.len_chars();
        let slice = rope.slice(1..len - 1);
        bench.iter(|| {
            slice.slice(..);
        })
    });
}

fn ptable_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_slice");

    group.bench_function("slice", |bench| {
        let ptable = PieceTable::new(TEXT);
        let len = ptable.len();
        bench.iter(|| {
            let mut start = random::<u64>() as usize % (len + 1);
            let mut end = random::<u64>() as usize % (len + 1);
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            ptable.slice(start..end);
        })
    });

    group.bench_function("slice_small", |bench| {
        let ptable = PieceTable::new(TEXT);
        let len = ptable.len();
        bench.iter(|| {
            let mut start = random::<u64>() as usize % (len + 1);
            if start > (len - 65) {
                start = len - 65;
            }
            let end = start + 64;
            ptable.slice(start..end);
        })
    });

    group.bench_function("slice_from_small_ptable", |bench| {
        let ptable = PieceTable::new(SMALL_TEXT);
        let len = ptable.len();
        bench.iter(|| {
            let mut start = random::<u64>() as usize % (len + 1);
            let mut end = random::<u64>() as usize % (len + 1);
            if start > end {
                std::mem::swap(&mut start, &mut end);
            }
            ptable.slice(start..end);
        })
    });

    group.bench_function("slice_whole_ptable", |bench| {
        let ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            ptable.slice(0..TEXT.len());
        })
    });

    group.bench_function("slice_whole_slice", |bench| {
        let ptable = PieceTable::new(TEXT);
        let len = ptable.len();
        let slice = ptable.slice(1..len - 1);
        bench.iter(|| {
            slice.slice(0..TEXT.len());
        })
    });
}

fn rope_len(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_len");

    group.bench_function("len_chars", |bench| {
        let rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.len_chars();
        })
    });

    group.bench_function("len_bytes", |bench| {
        let rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.len_bytes();
        })
    });

    group.bench_function("len_lines", |bench| {
        let rope = Rope::from_str(TEXT);
        bench.iter(|| {
            rope.len_lines();
        })
    });
}

fn string_len(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_len");

    group.bench_function("len", |bench| {
        let string = String::from(TEXT);
        bench.iter(|| {
            string.len();
        })
    });

    group.bench_function("chars_count", |bench| {
        let string = String::from(TEXT);
        bench.iter(|| {
            string.chars().count();
        })
    });

    group.bench_function("lines_count", |bench| {
        let string = String::from(TEXT);
        bench.iter(|| {
            string.lines().count();
        })
    });
}

fn ptable_len(c: &mut Criterion) {
    let mut group = c.benchmark_group("ptable_len");

    group.bench_function("len", |bench| {
        let ptable = PieceTable::new(TEXT);
        bench.iter(|| {
            ptable.len();
        })
    });
}

//----

criterion_group!(
    benches,
    rope_slice,
    ptable_slice,
    rope_len,
    string_len,
    ptable_len
);
criterion_main!(benches);
