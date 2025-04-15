use std::{
    collections::HashMap,
    fmt::format,
    io::{Cursor, Write},
    sync::{atomic::AtomicU16, RwLock},
    time::Instant,
};

use ftfrs::{Record, StringRecord};
use rustc_hash::FxHashMap;

fn main() {
    let n = 100_000_000;
    let k = 32_768;
    // let n = 10_000;
    let v = vec![0; 8 * 2 * n];
    let mut c = Cursor::new(v);
    let mut strs = Vec::new();

    let mut map: RwLock<FxHashMap<String, u16>> = RwLock::new(FxHashMap::default());
    for i in 0..k {
        let s = format!("foo_{i}");
        strs.push(s.clone());
        // map.insert(s, i as u16);
    }
    let mut x = AtomicU16::new(0);
    let t = Instant::now();

    for i in 0..n {
        let r = i & ((1 << 8) - 1);
        let mut map = map.write().unwrap();
        let idx = if let Some(idx) = map.get(&strs[i % k]) {
            *idx
        } else {
            let x = x.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            map.insert(strs[i % k].clone(), x);
            let str_record = Record::create_string(x, strs[i % k].clone());
            str_record.write(&mut c).unwrap();
            x
        };
        let idx2 = if let Some(idx) = map.get(&strs[(i + 1) % k]) {
            *idx
        } else {
            let x = x.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            map.insert(strs[(i + 1) % k].clone(), x);
            let str_record = Record::create_string(x, strs[i % k].clone());
            str_record.write(&mut c).unwrap();
            x
        };

        let rec = ftfrs::Record::create_duration_begin_event(
            0,
            ftfrs::ThreadRef::Ref(r as u8),
            ftfrs::StringRef::Ref(idx),
            ftfrs::StringRef::Ref(idx2),
            Vec::new(),
        );
        rec.write(&mut c).unwrap();
        // baseline(&mut c);
    }

    println!("{} ns/write", t.elapsed().as_nanos() as usize / n);
}

fn baseline<W: Write>(w: &mut W) {
    w.write_all(&0_u64.to_ne_bytes()).unwrap();
    w.write_all(&1_u64.to_ne_bytes()).unwrap();
}
