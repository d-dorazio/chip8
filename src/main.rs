use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use structopt::StructOpt;

const KEY_MAPPINGS: [Keycode; 16] = [
    Keycode::X,
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::Z,
    Keycode::C,
    Keycode::Num4,
    Keycode::R,
    Keycode::F,
    Keycode::V,
];

#[derive(Debug, StructOpt)]
struct App {
    /// Game rom to play
    #[structopt(parse(from_os_str))]
    rom: PathBuf,

    /// Frequency of the emulator
    #[structopt(short = "f", long = "frequency", default_value = "500")]
    freq: usize,
}

fn main() {
    let app = App::from_args();

    let mut rom = File::open(app.rom).expect("cannot open rom");

    let mut prog = vec![];
    rom.read_to_end(&mut prog).expect("cannot read rom");

    let mut chip8 = chip8::Chip8::with_program(rand::thread_rng(), &prog).unwrap();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(env!("CARGO_PKG_NAME"), 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return,
                Event::KeyDown {
                    keycode: Some(kc), ..
                } => {
                    let hex_key = KEY_MAPPINGS.iter().position(|m| *m == kc);
                    if let Some(hex_key) = hex_key {
                        chip8.keypress(hex_key as u8);
                    }
                }
                Event::KeyUp {
                    keycode: Some(kc), ..
                } => {
                    let hex_key = KEY_MAPPINGS.iter().position(|m| *m == kc);
                    if let Some(hex_key) = hex_key {
                        chip8.keyrelease(hex_key as u8);
                    }
                }
                _ => {}
            }
        }

        for _ in 0..app.freq / 60 {
            chip8.emulate_cycle();
        }

        canvas.clear();

        for (y, x, p) in chip8.pixels() {
            if *p == 1 {
                canvas.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
            } else {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
            }

            canvas
                .fill_rect(sdl2::rect::Rect::new(x as i32 * 10, y as i32 * 10, 10, 10))
                .unwrap();
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        chip8.decrease_timers();
    }
}
