mod beeper;

use std::cell::RefCell;
use std::rc::Rc;

use cfg_if::cfg_if;

use wasm_bindgen;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::KeyboardEvent;

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

// const MAZE: &[u8] = include_bytes!("../../../games/MAZE");
// const TETRIS: &[u8] = include_bytes!("../../../games/TETRIS");
const PONG: &[u8] = include_bytes!("../../../games/PONG");

const FREQ: usize = 500;

const KEY_MAPPINGS: [&str; 16] = [
    "x", "1", "2", "3", "q", "w", "e", "a", "s", "d", "z", "c", "4", "r", "f", "v",
];

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let document = window().document().expect("should have a Document");

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    canvas.set_width(640);
    canvas.set_height(320);

    document
        .get_element_by_id("game-container")
        .unwrap()
        .append_child(&canvas)?;

    let chip8 = chip8::Chip8::with_program(rand::rngs::OsRng::new().unwrap(), PONG).unwrap();
    let chip8 = Rc::new(RefCell::new(chip8));

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.fill_rect(0.0, 0.0, 640.0, 320.0);

    {
        let chip8 = chip8.clone();
        let on_key_press = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            let hex_key = KEY_MAPPINGS.iter().position(|m| *m == e.key());

            if let Some(hex_key) = hex_key {
                chip8.borrow_mut().keypress(hex_key as u8);
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        document.set_onkeydown(Some(on_key_press.as_ref().unchecked_ref()));
        on_key_press.forget();
    }

    {
        let chip8 = chip8.clone();
        let on_key_release = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            let hex_key = KEY_MAPPINGS.iter().position(|m| *m == e.key());

            if let Some(hex_key) = hex_key {
                chip8.borrow_mut().keyrelease(hex_key as u8);
            }
        }) as Box<dyn FnMut(KeyboardEvent)>);

        document.set_onkeyup(Some(on_key_release.as_ref().unchecked_ref()));
        on_key_release.forget();
    }

    let beeper = beeper::Beeper::new().unwrap();

    register_animation_frame_loop(move || {
        let mut chip8 = chip8.borrow_mut();

        for _ in 0..FREQ / 60 {
            chip8.emulate_cycle();
        }

        for (y, x, p) in chip8.pixels() {
            let style = if *p == 1 {
                JsValue::from_str("black")
            } else {
                JsValue::from_str("white")
            };

            context.set_fill_style(&style);

            context.fill_rect(x as f64 * 10.0, y as f64 * 10.0, 10.0, 10.0);
        }

        if chip8.beep() {
            beeper.resume().unwrap();
        } else {
            beeper.pause().unwrap();
        }

        chip8.decrease_timers();
    });

    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn register_animation_frame_loop<F: FnMut() + 'static>(mut fun: F) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        fun();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
