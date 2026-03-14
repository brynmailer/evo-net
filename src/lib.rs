use log::*;
use screeps_arena::game;
use wasm_bindgen::prelude::*;

mod logging;

fn setup() {
    logging::setup_logging(logging::Info);
}

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = wasm_loop)]
pub fn tick() {
    let tick = game::utils::get_ticks();

    if tick == 1 {
        setup();
    }

    info!("Current tick: {tick}");
}
