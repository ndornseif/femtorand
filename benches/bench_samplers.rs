use criterion::{Criterion, criterion_group, criterion_main};
use femtorand::*;
use std::hint::black_box;

pub fn criterion_benchmark(c: &mut Criterion) {
    // Test values: 1,2,3,4,5,8,9,10,15,16,17,31,32,33 and powers of two up to 1<<16
    let mut test_values = vec![1, 2, 3, 10, 20, u32::MAX as usize, 1 << 20];

    for &max_int in &test_values {
        let group_name = format!("Fill size: {}", max_int);
        let mut group = c.benchmark_group(&group_name);
        let mut prng_l = Lehmer64::new(0xDEADBEEF);
        let mut prng_w = WyRand::new(0xDEADBEEF);
        let mut dest_u8 = vec![0_u8; max_int];
        group.bench_function("u64_range", |b| {
            b.iter(|| prng_w.next_lim_u64(max_int as u64))
        });

        group.bench_function("u32_range", |b| {
            b.iter(|| prng_w.next_lim_u32(max_int as u32))
        });

        group.bench_function("u32_range_fase", |b| {
            b.iter(|| prng_w.generate_int_lim::<u32>(max_int as u32))
        });

        group.finish();
    }
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
