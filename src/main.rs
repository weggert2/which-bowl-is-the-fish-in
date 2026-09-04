mod app;
mod config;

use macroquad::prelude::*;

#[macroquad::main("Which Bowl is the Fish In?")]
async fn main() {
    let mut app = app::WhichBowlApp::new().await;

    loop {
        app.update();
        app.draw();

        next_frame().await;
    }
}
