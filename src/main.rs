#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;

use macroquad::prelude::*;
use which_bowl::config::GameConfig;

mod generated_window_icon {
    include!(concat!(env!("OUT_DIR"), "/window_icon.rs"));
}

fn window_conf() -> macroquad::conf::Conf {
    let config = GameConfig::default();
    macroquad::conf::Conf {
        miniquad_conf: macroquad::miniquad::conf::Conf {
            window_title: "Which Bowl is the Fish In?".to_string(),
            window_width: config.window_width as i32,
            window_height: config.window_height as i32,
            window_resizable: false,
            icon: Some(macroquad::miniquad::conf::Icon {
                small: generated_window_icon::APP_ICON_16,
                medium: generated_window_icon::APP_ICON_32,
                big: generated_window_icon::APP_ICON_64,
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = app::WhichBowlApp::new().await;

    loop {
        app.update().await;
        app.draw();

        next_frame().await;
    }
}
