use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct App {
    /// Game rom to play
    #[structopt(parse(from_os_str))]
    rom: PathBuf,
}

fn main() {
    let app = App::from_args();

    let mut rom = File::open(app.rom).expect("cannot open rom");

    let mut prog = vec![];
    rom.read_to_end(&mut prog).expect("cannot read rom");

    let mut chip8 = chip8::Chip8::with_program(rand::thread_rng(), &prog).unwrap();

    for _ in 0..10_000 {
        chip8.emulate_cycle();
    }
}
