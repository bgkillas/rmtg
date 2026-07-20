extern crate test;
use crate::image::parse_bytes;
use std::hint::black_box;
use test::Bencher;
#[bench]
fn bench_write_data(bencher: &mut Bencher) {
    let bytes = include_bytes!("../../assets/back.jpg");
    bencher.iter(|| black_box(parse_bytes(bytes).unwrap()))
}
