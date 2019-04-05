mod render_hash_2d_cpu;

use render_hash_2d_cpu::render_hash_2d_cpu;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::Write;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
fn usage() {
    println!("Usage: cargo run [hash_type]");
    println!("hash types:");
    println!("  --cpu - run render_hash on CPU");
}

fn colored_print(msg: &str, color: Color, background_color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    stdout.set_color(ColorSpec::new().set_bg(Some(background_color)));
    stdout.set_color(ColorSpec::new().set_fg(Some(color)));

    write!(&mut stdout, "{}", msg).unwrap();

    stdout.set_color(ColorSpec::new().set_bg(Some(Color::Black)));
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
}

fn cpu_mine(tx_dir: &str) {
    let mut msg = [0; 80];

    let start_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut last_stat_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let mut hash_count = 0;

    let end = u32::pow(2, 16);

    for i in 0..end {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        if now - last_stat_ts > 1000 {
            let hash_rate = hash_count as f64/(now - last_stat_ts) as f64 * 1000 as f64;
            println!("hashrate: {}", hash_rate);
            last_stat_ts = now;
            hash_count = 0;
        }
        hash_count += 1;

        msg[10] = i as u8;

        let hash = render_hash_2d_cpu(&msg, &tx_dir, false, false);
        colored_print("Mining..\n", Color::Rgb(0x20, 0xc2, 0x0e), Color::Black);
        if hash[0] == 0 {
            colored_print("Mining is finished. ", Color::Red, Color::Green);
            println!("Final hash: {:?}", hash);
            break;
        }
    }
}

fn main() {
	let dir: &str = "./tex/";
    
    let hash_types: Vec<String> = vec!["--cpu", "--gpu"].iter().map(|x| x.to_string()).collect();

    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && hash_types.contains(&args[1]) {
        if args[1] == "--cpu" {
            cpu_mine("./tex/");
        }
    }
    else {
        usage();
    }
}