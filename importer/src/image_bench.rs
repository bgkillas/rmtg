extern crate test;
use crate::image::parse_bytes;
use std::hint::black_box;
use test::Bencher;
#[bench]
fn bench_back(bencher: &mut Bencher) {
    let bytes = include_bytes!("../../assets/back.png");
    bencher.iter(|| black_box(parse_bytes(bytes).unwrap()))
}
