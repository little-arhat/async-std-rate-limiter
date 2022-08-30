mod rate_limiter;
mod server;
mod symbols;

use std::env;
use std::hash::{Hash, Hasher};
use std::process;

const EX_USAGE: i32 = 64; //#include <sysexits.h>
const EX_OK: i32 = 0;

const USAGE: &'static str = "Usage: ./app <PARTITIONS:[1;26]> <LIMIT:[1; 200]>";

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "-h" {
        show_usage();
        process::exit(EX_OK);
    } else if args.len() != 3 {
        exit_with_msg("Invalid input! No args provided!");
    }
    let num_partitions = args[1]
        .parse::<u8>()
        .ok()
        .filter(|v| v >= &1)
        .filter(|v| v <= &26)
        .ok_or("Expects PARTITIONS to be int in [1; 26]")?;

    let limit = args[2]
        .parse::<u8>()
        .ok()
        .filter(|v| v >= &1)
        .filter(|v| v <= &200)
        .and_then(|v| core::num::NonZeroU8::new(v))
        .ok_or("Expects LIMIT to be int in [1; 200]")?;

    println!("Generated partitions:");
    let partitioned_symbols = symbols::generate(num_partitions, seed());
    for partition in partitioned_symbols.iter() {
        let mut for_debug = partition.clone();
        for_debug.sort();
        println!("{:?}", for_debug);
    }

    let partitioned_indices = partitioned_symbols
        .into_iter()
        .map(|p| {
            p.into_iter()
                .map(|s| symbols::to_index(s).expect("Expect to generate symbols from range"))
                .collect()
        })
        .collect::<Vec<Vec<usize>>>();

    let server_address = address();
    println!(
        "Starting server at {}; limit: {} request per partition per second",
        server_address, limit
    );
    let serve_task = server::serve(partitioned_indices, server_address, limit);
    let jh = async_std::task::spawn(serve_task);
    async_std::task::block_on(jh).map_err(|e| format!("Error in server: {:?}", e))?;

    Ok(())
}

fn address() -> String {
    env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:31337".to_owned())
}

fn seed() -> Option<u64> {
    env::var("SERVER_SEED").ok().map(|v| {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    })
}

fn show_usage() {
    println!("{}", USAGE);
}

fn exit_with_msg(err: &str) {
    eprintln!("{}", err);
    eprintln!("{}", USAGE);
    process::exit(EX_USAGE);
}
