#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let disable_graphics = option_env!("AF_DISABLE_GRAPHICS");
    if disable_graphics.is_none() || !disable_graphics.unwrap().eq("1"){
        panic!("Please set AF_DISABLE_GRAPHICS=1")
    }
    let app = mosaic::MosaicApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("Mosaic",native_options, Box::new(|_| Box::new(app)));
}
