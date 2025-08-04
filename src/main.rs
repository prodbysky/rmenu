use std::collections::HashSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const MAX_SUGGESTIONS: usize = 5;
const FONT_SIZE: i32 = 32;

use raylib::math::Rectangle;
use raylib::math::Vector2;
use raylib::text::RaylibFont;
use raylib::{color::Color, drawing::RaylibDraw, ffi::KeyboardKey};

fn main() {
    let (mut handle, thread) = raylib::RaylibBuilder::default()
        .size(640, 480)
        .title("rmenu")
        .vsync()
        .msaa_4x()
        .transparent()
        .build();

    let font = handle
        .load_font_ex(&thread, "iosevka-regular.ttf", FONT_SIZE, None)
        .unwrap();

    let single_char_size = font.measure_text("a", FONT_SIZE as f32, 0.0);
    let single_element_height = single_char_size.y * 2.0;
    let pad_left = single_char_size.x;
    let pad_top = single_char_size.y / 2.0;

    let margin_y = single_char_size.y / 4.0;

    handle.set_window_size(
        640,
        12 * single_char_size.y as i32 + (pad_top * 2.0) as i32 + margin_y as i32,
    );

    let mut buffer = String::new();
    let mut pos = 0;

    let mut selected_item = 0;

    let available = get_executables_on_path();

    while !handle.window_should_close() {
        let mut relevant = available
            .iter()
            .filter(|exe_name| exe_name.starts_with(&buffer))
            .collect::<Vec<_>>();
        relevant.sort_by(|l, r| (l.len() as i32 - r.len() as i32).cmp(&0));
        let relevant = relevant.iter().take(MAX_SUGGESTIONS).collect::<Vec<_>>();

        if let Some(c) = handle.get_char_pressed() {
            buffer.push(c);
            pos += 1;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
            Command::new("alacritty")
                .arg("-e")
                .arg(relevant[selected_item])
                .spawn()
                .unwrap();
            break;
        }
        if handle.is_key_down(KeyboardKey::KEY_BACKSPACE) {
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
        if handle.is_key_pressed(KeyboardKey::KEY_UP) {
            if selected_item > 0 {
                selected_item -= 1;
            }
        }
        if handle.is_key_pressed(KeyboardKey::KEY_DOWN) {
            if selected_item < MAX_SUGGESTIONS - 1 {
                selected_item += 1;
            }
        }
        let mut draw = handle.begin_drawing(&thread);
        draw.clear_background(Color::get_color(0x18181822));
        let mut y_pos = pad_top;
        draw.draw_rectangle_rounded(
            Rectangle::new(
                pad_left,
                pad_top,
                640.0 - pad_left * 2.0,
                single_element_height,
            ),
            0.4,
            12,
            Color::get_color(0xffffff20),
        );
        y_pos += single_char_size.y / 2.0;
        draw.draw_text_ex(
            &font,
            &buffer,
            Vector2::new(pad_left + pad_left / 2.0, y_pos),
            FONT_SIZE as f32,
            0.0,
            Color::RAYWHITE,
        );
        y_pos += single_char_size.y * 1.5;

        for (i, name) in relevant.iter().enumerate() {
            if i == selected_item {
                draw.draw_rectangle_rounded(
                    Rectangle::new(
                        pad_left,
                        single_element_height * (i + 1) as f32 + pad_top + margin_y,
                        640.0 - pad_left * 2.0,
                        single_element_height,
                    ),
                    0.4,
                    12,
                    Color::get_color(0xffffff20),
                );
            }
            y_pos += single_char_size.y / 2.0;
            draw.draw_text_ex(
                &font,
                &name,
                Vector2::new(pad_left + pad_left / 2.0, y_pos + margin_y),
                FONT_SIZE as f32,
                0.0,
                Color::RAYWHITE,
            );
            y_pos += single_char_size.y * 1.5;
        }
    }
}

fn is_executable(path: &PathBuf) -> bool {
    if !path.is_file() {
        return false;
    }
    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        permissions.mode() & 0o111 != 0
    } else {
        false
    }
}

fn get_executables_on_path() -> HashSet<String> {
    let mut executables = HashSet::new();

    if let Some(paths) = env::var_os("PATH") {
        for path in env::split_paths(&paths) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if is_executable(&path) {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            executables.insert(file_name.to_string());
                        }
                    }
                }
            }
        }
    }

    executables
}
