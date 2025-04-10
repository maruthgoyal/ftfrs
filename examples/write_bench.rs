use std::{
    io::{Cursor, Write},
    time::Instant,
};

fn main() {
    let n = 100_000_000;
    let v = vec![0; 8 * 2 * n];
    let mut c = Cursor::new(v);
    let t = Instant::now();
    let rec = ftfrs::Record::create_duration_begin_event(
        0,
        ftfrs::ThreadRef::Ref(1),
        ftfrs::StringRef::Ref(1),
        ftfrs::StringRef::Ref(2),
        Vec::new(),
    );
    for _ in 0..n {
        rec.write(&mut c).unwrap();
        // baseline(&mut c);
    }
    println!("{} ns/write", t.elapsed().as_nanos() as usize / n);
}

fn baseline<W: Write>(w: &mut W) {
    w.write_all(&0_u64.to_ne_bytes()).unwrap();
    w.write_all(&1_u64.to_ne_bytes()).unwrap();
}
