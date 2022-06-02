use criterion::{criterion_group, criterion_main, Criterion};
use flux_error::filesystem::FileId;
use flux_parser::parse;

const INPUT_FILE: &str = r#"
type Bar u16

type Foo struct {
  i32 x;
  pub u17 y;
  pub mut Bar z;
  mut f64 a;
}

fn main(mut u32 argc, i6 test) -> Foo {
  let x = 10;
  i32 y = x;
}
"#;

fn criterion_benchmark(c: &mut Criterion) {
	c.bench_function("parse basic function with type decls", |b| {
		b.iter(|| parse(INPUT_FILE, FileId(0)))
	});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
