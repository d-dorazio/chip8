use wasm_bindgen::prelude::*;

use web_sys::{AudioContext, GainNode, OscillatorNode, OscillatorType};

// oscillator and gain are stored in the struct to maintain them alive
#[allow(dead_code)]
pub struct Beeper {
    audio_ctx: AudioContext,
    oscillator: OscillatorNode,
    gain: GainNode,
}

impl Beeper {
    pub fn new() -> Result<Self, JsValue> {
        let audio_ctx = AudioContext::new()?;

        let oscillator = audio_ctx.create_oscillator()?;
        let gain = audio_ctx.create_gain()?;

        oscillator.set_type(OscillatorType::Sine);
        oscillator.frequency().set_value(440.0);

        oscillator.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&audio_ctx.destination())?;

        oscillator.start()?;
        audio_ctx.suspend().map(|_| ())?;

        Ok(Beeper {
            audio_ctx,
            oscillator,
            gain,
        })
    }

    pub fn resume(&self) -> Result<(), JsValue> {
        self.audio_ctx.resume().map(|_| ())
    }

    pub fn pause(&self) -> Result<(), JsValue> {
        self.audio_ctx.suspend().map(|_| ())
    }
}
