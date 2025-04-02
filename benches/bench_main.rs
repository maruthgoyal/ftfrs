use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, black_box};
use ftfrs::{Archive, Record, StringRef, ThreadRef, Argument};
use std::io::{Cursor, Sink};

pub fn generate_sample_trace(size: usize, interned_percentage: usize) -> Vec<u8> {
    let archive = create_mixed_archive(size, interned_percentage);
    let mut buffer = Vec::new();
    archive.write(&mut buffer).unwrap();
    buffer
}

pub fn create_mixed_archive(num_events: usize, interned_pct: usize) -> Archive {
    let mut archive = Archive {
        records: vec![Record::create_magic_number()]
    };
    
    archive.records.push(Record::create_initialization(1_000_000));
    
    archive.records.push(Record::create_thread(1, 0x1234, 0x5678));
    
    let category_ref = 1;
    let name_ref = 2;
    archive.records.push(Record::create_string(category_ref, "category".to_string()));
    archive.records.push(Record::create_string(name_ref, "event_name".to_string()));
    
    let arg_names = [
        "duration_ms", "size_bytes", "count", "success", "status_code", 
        "process_id", "thread_id", "sequence", "memory", "timestamp"
    ];
    
    for (i, name) in arg_names.iter().enumerate() {
        archive.records.push(Record::create_string(10 + i as u16, name.to_string()));
    }
    
    // Add events with mixed string handling
    for i in 0..num_events {
        let timestamp = i as u64 * 100;
        
        let use_interned = (i * 100 / num_events) < interned_pct;
        
        let category = if use_interned {
            StringRef::Ref(category_ref)
        } else {
            StringRef::Inline("category".to_string())
        };
        
        let name = if use_interned {
            StringRef::Ref(name_ref)
        } else {
            StringRef::Inline("event_name".to_string())
        };
        
        let mut args = Vec::new();
        
        let num_args = i % 4;
        for j in 0..num_args {
            let arg_idx = j % arg_names.len();
            let arg_name = if use_interned && j % 2 == 0 {
                StringRef::Ref(10 + arg_idx as u16)
            } else {
                StringRef::Inline(arg_names[arg_idx].to_string())
            };
            
            match j % 3 {
                0 => args.push(Argument::Int64(arg_name, i as i64)),
                1 => args.push(Argument::UInt64(arg_name, i as u64)),
                _ => args.push(Argument::Float(arg_name, i as f64)),
            }
        }
        
        let event = match i % 4 {
            0 => Record::create_instant_event(
                timestamp, ThreadRef::Ref(1), category, name, args
            ),
            1 => Record::create_counter_event(
                timestamp, ThreadRef::Ref(1), category, name, args, i as u64
            ),
            2 => Record::create_duration_begin_event(
                timestamp, ThreadRef::Ref(1), category, name, args
            ),
            _ => Record::create_duration_end_event(
                timestamp, ThreadRef::Ref(1), category, name, args
            ),
        };
        
        archive.records.push(event);
    }
    
    archive
}


pub fn bench_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_read");
    
    for events in [10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        let buffer = generate_sample_trace(*events, 50);
        
        group.bench_with_input(BenchmarkId::from_parameter(events), &buffer, |b, buffer| {
            b.iter(|| {
                let mut cursor = Cursor::new(buffer);
                let archive = black_box(Archive::read(&mut cursor).unwrap());
                black_box(archive)
            });
        });
    }
    
    group.finish();
}


pub fn bench_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("archive_write");
    
    for events in [10, 100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        let archive = create_mixed_archive(*events, 50);
        
        group.bench_with_input(BenchmarkId::from_parameter(events), events, |b, _| {
            b.iter(|| {
                let mut sink = Sink::default();
                black_box(archive.write(&mut sink).unwrap());
            });
        });
    }
    
    group.finish();
}


pub fn bench_string_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_handling");
    
    for string_size in [8, 16, 32, 64, 128, 256].iter() {
        let test_string = "X".repeat(*string_size);
        
        group.bench_with_input(
            BenchmarkId::new("inline_string", string_size), 
            &test_string, 
            |b, s| {
                b.iter(|| {
                    let record = Record::create_instant_event(
                        100,
                        ThreadRef::Ref(1),
                        StringRef::Inline("category".to_string()),
                        StringRef::Inline(s.clone()),
                        Vec::new(),
                    );
                    
                    let mut sink = Sink::default();
                    black_box(record.write(&mut sink).unwrap())
                });
            }
        );
        
        group.bench_function(
            BenchmarkId::new("string_reference", string_size), 
            |b| {
                b.iter_batched(
                    || {
                        let event_record = Record::create_instant_event(
                            100,
                            ThreadRef::Ref(1),
                            StringRef::Inline("category".to_string()),
                            StringRef::Ref(1),
                            Vec::new(),
                        );
                        
                        event_record
                    },
                    | event_record| {
                        let mut sink = Sink::default();
                        black_box(event_record.write(&mut sink).unwrap());
                    },
                    criterion::BatchSize::SmallInput,
                );
            }
        );
    }
    
    group.finish();
}


pub fn bench_mixed_workloads(c: &mut Criterion) {
    bench_string_interning_ratio(c);
    bench_event_argument_count(c);
    bench_throughput(c);
}

fn bench_string_interning_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning_ratio");
    
    // Define different ratios of interned vs inline strings (percentage interned)
    for interned_pct in [0, 25, 50, 75, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(interned_pct),
            interned_pct,
            |b, &interned_pct| {
                b.iter_batched(
                    || {
                        create_mixed_archive(1000, interned_pct)
                    },
                    |archive| {
                        let mut sink = Sink::default();
                        black_box(archive.write(&mut sink).unwrap());
                    },
                    criterion::BatchSize::SmallInput,
                );
            }
        );
    }
    
    group.finish();
}

fn bench_event_argument_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_argument_count");
    
    for arg_count in [0, 1, 2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(arg_count),
            arg_count,
            |b, &arg_count| {
                b.iter(|| {
                    let mut args = Vec::new();
                    for i in 0..arg_count {
                        args.push(Argument::Int64(
                            StringRef::Inline(format!("arg_{}", i)),
                            i as i64
                        ));
                    }
                    
                    let record = Record::create_instant_event(
                        100,
                        ThreadRef::Ref(1),
                        StringRef::Inline("category".to_string()),
                        StringRef::Inline("event_name".to_string()),
                        args,
                    );
                    
                    let mut sink = Sink::default();
                    black_box(record.write(&mut sink).unwrap());
                });
            }
        );
    }
    
    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    
    for event_count in [100, 1000, 10000].iter() {
        let archive = create_mixed_archive(*event_count, 50);  // 50% interned strings
        
        let mut buffer = Vec::new();
        archive.write(&mut buffer).unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("write", event_count),
            &archive,
            |b, archive: &Archive| {
                b.iter(|| {
                    let mut sink = Sink::default();
                    black_box(archive.write(&mut sink).unwrap());
                });
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("read", event_count),
            &buffer,
            |b, buffer| {
                b.iter(|| {
                    let mut cursor = Cursor::new(buffer);
                    black_box(Archive::read(&mut cursor).unwrap());
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_read,
    bench_write,
    bench_string_handling,
    bench_mixed_workloads
);
criterion_main!(benches);