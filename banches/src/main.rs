mod structs;
mod string_tests;
mod globre_tests;
mod tracking_allocator;

use std::hint::black_box;
use std::time::Instant;
use structs::TestTrait;
use tracking_allocator::AllocStats;

pub struct Entry {
    pub name:        &'static str,
    pub description: &'static str,
    pub run:         fn(count: usize, repeats: usize),
}
impl Entry {
    const fn new<T: TestTrait>() -> Self {
        Self {
            name: T::NAME,
            description: T::DESCRIPTION,
            run: |count, repeats| run::<T>(count, repeats),
        }
    }
}

static ENTRIES: &[Entry] = &[
    Entry::new::<string_tests::IsOneOf>(),
    Entry::new::<string_tests::StartsWith>(),
    Entry::new::<string_tests::EndsWith>(),
    Entry::new::<string_tests::Contains>(),
    Entry::new::<globre_tests::GlobREMatch>(),
];


fn run<T: TestTrait>(count: usize, repeats: usize) {
    let before_init = AllocStats::now();
    let mut test = T::init();
    let after_init = AllocStats::now();
    let mut sum = 0u128;
    println!("Running test '{}' — {}", T::NAME, T::DESCRIPTION);
    for i in 0..repeats {
        print!("  [{}/{}] ", i + 1, repeats);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let start = Instant::now();
        let _: () = test.run_test(black_box(count));
        black_box(());
        let duration = start.elapsed();
        println!("{} ms", duration.as_millis());
        sum += duration.as_millis();
    }
    println!("  Average: {} ms\n", sum / repeats as u128);
    println!("  Memory usage: {} bytes", after_init.bytes - before_init.bytes);
}

fn usage(prog: &str) {
    eprintln!("Usage:\n{prog} LIST\n{prog} RUN <count> <repeats>\n{prog} RUN <count> <repeats> <search>\n\nExamples:\n{prog} LIST\n{prog} RUN 100 10\n{prog} RUN 100 10 isoneof");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let prog = args[0].as_str();

    if args.len() < 2 {
        usage(prog);
        return;
    }

    match args[1].to_uppercase().as_str() {
        "LIST" => {
            println!("{:<30}  DESCRIPTION", "NAME");
            println!("{}", "─".repeat(72));
            for e in ENTRIES {
                println!("{:<30}  {}", e.name, e.description);
            }
        }

        // ── RUN ───────────────────────────────────────────────────────────
        "RUN" => {
            if args.len() < 4 {
                usage(prog);
                std::process::exit(1);
            }

            let count: usize = args[2].parse().unwrap_or_else(|_| {
                eprintln!("Error: <count> must be a positive integer.");
                std::process::exit(1);
            });
            let repeats: usize = args[3].parse().unwrap_or_else(|_| {
                eprintln!("Error: <repeats> must be a positive integer.");
                std::process::exit(1);
            });

            // Optional search filter (arg index 4)
            let filter: Option<String> = args.get(4).map(|s| s.to_lowercase());

            let matches: Vec<&Entry> = ENTRIES
                .iter()
                .filter(|e| match &filter {
                    None => true,
                    Some(q) => e.name.to_lowercase().contains(q.as_str())
                           || e.description.to_lowercase().contains(q.as_str()),
                })
                .collect();

            if matches.is_empty() {
                eprintln!(
                    "No tests matched '{}'.",
                    filter.unwrap_or_default()
                );
                std::process::exit(1);
            }

            for entry in matches {
                (entry.run)(count, repeats);
            }
        }

        _ => {
            usage(prog);
            std::process::exit(1);
        }
    }
}