#[path = "../src/steg/mod.rs"]
mod steg;

use criterion::{criterion_group, criterion_main, Criterion, Throughput, BenchmarkId};
use std::fs;

fn encodebench(c: &mut Criterion) {
    let message_path = String::from("bible.txt");
    let metadata = fs::metadata(message_path).expect("Our benchmark file should exist");
    let message_size = metadata.len() as u64;

    let mut group = c.benchmark_group("Encode");
    group.sample_size(10);
    group.throughput(criterion::Throughput::Bytes(message_size));

    for n_threads in 1..20 {
        group.bench_with_input(
            BenchmarkId::new("Num threads", n_threads),
            &n_threads,
            |b, &n_threads| b.iter(|| { steg::encode_directory(n_threads,
                  String::from("bible.txt"), 
                  String::from("aux/in/"),
                  String::from("aux/out/benchOut/"))}));
    }
}

fn decodebench(c: &mut Criterion) {
    let message_path = String::from("bible.txt");
    let metadata = fs::metadata(message_path).expect("Our benchmark file should exist");
    let message_size = metadata.len() as u64;

    let mut group = c.benchmark_group("Decode");
    group.sample_size(10);
    group.throughput(criterion::Throughput::Bytes(message_size));

    for n_threads in 1..20 {
        group.bench_with_input(
            BenchmarkId::new("Num threads", n_threads),
            &n_threads,
            |b, &n_threads| b.iter(|| { steg::decode_directory(n_threads,
                  String::from("aux/out/benchOut/"))}));
    }
}

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(10)
}

criterion_group!(benches, decodebench);//encodebench);
criterion_main!(benches);

