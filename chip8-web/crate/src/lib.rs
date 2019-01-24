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

const GAMES: [(&str, &[u8]); 23] = [
    ("15PUZZLE", include_bytes!("../../../games/15PUZZLE")),
    ("BLINKY", include_bytes!("../../../games/BLINKY")),
    ("BLITZ", include_bytes!("../../../games/BLITZ")),
    ("BRIX", include_bytes!("../../../games/BRIX")),
    ("CONNECT4", include_bytes!("../../../games/CONNECT4")),
    ("GUESS", include_bytes!("../../../games/GUESS")),
    ("HIDDEN", include_bytes!("../../../games/HIDDEN")),
    ("INVADERS", include_bytes!("../../../games/INVADERS")),
    ("KALEID", include_bytes!("../../../games/KALEID")),
    ("MAZE", include_bytes!("../../../games/MAZE")),
    ("MERLIN", include_bytes!("../../../games/MERLIN")),
    ("MISSILE", include_bytes!("../../../games/MISSILE")),
    ("PONG", include_bytes!("../../../games/PONG")),
    ("PUZZLE", include_bytes!("../../../games/PUZZLE")),
    ("SIERPINKSI", include_bytes!("../../../games/SIERPINKSI")),
    ("SYZYGY", include_bytes!("../../../games/SYZYGY")),
    ("TANK", include_bytes!("../../../games/TANK")),
    ("TETRIS", include_bytes!("../../../games/TETRIS")),
    ("TICTAC", include_bytes!("../../../games/TICTAC")),
    ("UFO", include_bytes!("../../../games/UFO")),
    ("VBRIX", include_bytes!("../../../games/VBRIX")),
    ("VERS", include_bytes!("../../../games/VERS")),
    ("WIPEOFF", include_bytes!("../../../games/WIPEOFF")),
];

const FREQ: usize = 500;

const KEY_MAPPINGS: [&str; 16] = [
    "x", "1", "2", "3", "q", "w", "e", "a", "s", "d", "z", "c", "4", "r", "f", "v",
];

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let document = window().document().expect("should have a Document");
    let select = document
        .create_element("select")?
        .dyn_into::<web_sys::HtmlSelectElement>()?;

    select.add_with_html_option_element(&web_sys::HtmlOptionElement::new_with_text("-")?)?;

    for (game, _) in &GAMES {
        let option = web_sys::HtmlOptionElement::new_with_text(game)?;
        select.add_with_html_option_element(&option)?;
    }

    let on_game_selected = Closure::wrap(Box::new(move |e: web_sys::Event| {
        web_sys::console::log_1(&e);
        let select = e
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlSelectElement>()
            .unwrap();
        let game_id = select.value();

        let game_rom = GAMES.iter().find(|(g, _)| g == &game_id);

        if let Some(game_rom) = game_rom {
            select.style().set_property("display", "none").unwrap();
            play_game(game_rom.1).unwrap();
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    select.set_onchange(Some(on_game_selected.as_ref().unchecked_ref()));
    on_game_selected.forget();

    document
        .get_element_by_id("game-container")
        .unwrap()
        .append_child(&select)?;

    Ok(())
}

fn play_game(rom: &[u8]) -> Result<(), JsValue> {
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

    let chip8 = chip8::Chip8::with_program(rand::rngs::OsRng::new().unwrap(), rom).unwrap();
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
            let style = if *p == 0 {
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
