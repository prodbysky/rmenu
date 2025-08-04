use std::process::Command;

// TODO:Launch terminal executables (to a pty or something)

use raylib::{color::Color, drawing::RaylibDraw, ffi::KeyboardKey};
fn main() {
    let (mut handle, thread) = raylib::RaylibBuilder::default()
        .size(640, 480)
        .title("rmenu")
        .build();

    let mut buffer = String::new();
    let mut pos = 0;

    while !handle.window_should_close() {
        if let Some(c) = handle.get_char_pressed() {
            buffer.push(c);
            pos += 1;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
            Command::new(buffer).spawn().unwrap();
            break;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            if pos != 0 {
                buffer.remove(pos - 1);
                pos -= 1;
            }
        }

        if handle.is_key_pressed(KeyboardKey::KEY_LEFT) {
            if pos > 0 {
                pos -= 1;
            }
        }
        if handle.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            if pos < buffer.len() {
                pos += 1;
            }
        }
        let mut draw = handle.begin_drawing(&thread);
        draw.clear_background(Color::get_color(0x181818ff));
        draw.draw_text(&buffer, 0, 0, 32, Color::RAYWHITE);
    }
}
