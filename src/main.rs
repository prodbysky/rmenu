use std::collections::HashSet;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

const MAX_SUGGESTIONS: usize = 5;
const FONT_SIZE: i32 = 32;

use raylib::RaylibHandle;
use raylib::RaylibThread;
use raylib::math::Rectangle;
use raylib::math::Vector2;
use raylib::text::Font;
use raylib::text::RaylibFont;
use raylib::{color::Color, drawing::RaylibDraw, ffi::KeyboardKey};

struct RMenu {
    h: RaylibHandle,
    t: RaylibThread,

    buffer: String,
    buffer_pos: usize,

    selected_item: usize,

    available_programs: HashSet<String>,

    ui: UI,
}

struct UI {
    font: Font,
}

impl UI {
    fn new(f: Font) -> Self {
        Self { font: f }
    }

    fn char_size(&self) -> Vector2 {
        self.font.measure_text("a", FONT_SIZE as f32, 0.0)
    }

    fn single_element_h(&self) -> f32 {
        self.char_size().y * 2.0
    }

    fn pad_left(&self) -> f32 {
        self.char_size().x
    }

    fn pad_top(&self) -> f32 {
        self.char_size().y / 2.0
    }

    fn margin_y(&self) -> f32 {
        self.pad_top() / 2.0
    }
}

impl RMenu {
    fn new() -> Self {
        let (mut handle, thread) = raylib::RaylibBuilder::default()
            .size(640, 480)
            .title("rmenu")
            .vsync()
            .msaa_4x()
            .transparent()
            .build();

        let ui = UI::new(
            handle
                .load_font_ex(
                    &thread,
                    "/usr/local/bin/iosevka-regular.ttf",
                    FONT_SIZE,
                    None,
                )
                .unwrap(),
        );

        handle.set_window_size(
            640,
            12 * ui.char_size().y as i32 + (ui.pad_top() * 2.0) as i32 + ui.margin_y() as i32,
        );
        Self {
            h: handle,
            t: thread,
            buffer: String::new(),
            buffer_pos: 0,
            selected_item: 0,
            available_programs: get_executables_on_path(),
            ui,
        }
    }
    fn get_relevant(&self) -> Vec<String> {
        let mut relevant = self
            .available_programs
            .iter()
            .filter(|exe_name| (*(*exe_name)).starts_with(&self.buffer))
            .map(|s| s.clone())
            .collect::<Vec<_>>();
        relevant.sort_by(|l, r| (l.len() as i32 - r.len() as i32).cmp(&0));
        relevant
            .iter()
            .take(MAX_SUGGESTIONS)
            .map(|s| s.clone())
            .collect::<Vec<_>>()
    }

    fn run(mut self) {
        while !self.h.window_should_close() {
            let relevant = self.get_relevant();

            if let Some(c) = self.h.get_char_pressed() {
                self.buffer.push(c);
                self.buffer_pos += 1;
            }
            if self.h.is_key_pressed(KeyboardKey::KEY_ENTER) {
                Command::new("alacritty")
                    .arg("-e")
                    .arg(&relevant[self.selected_item])
                    .spawn()
                    .unwrap();
                return;
            }
            if self.h.is_key_down(KeyboardKey::KEY_BACKSPACE) {
                if self.buffer_pos != 0 {
                    self.buffer.remove(self.buffer_pos - 1);
                    self.buffer_pos -= 1;
                }
            }

            if self.h.is_key_pressed(KeyboardKey::KEY_LEFT) {
                if self.buffer_pos > 0 {
                    self.buffer_pos -= 1;
                }
            }
            if self.h.is_key_pressed(KeyboardKey::KEY_RIGHT) {
                if self.buffer_pos < self.buffer.len() {
                    self.buffer_pos += 1;
                }
            }
            if self.h.is_key_pressed(KeyboardKey::KEY_UP) {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
            }
            if self.h.is_key_pressed(KeyboardKey::KEY_DOWN) {
                if self.selected_item < MAX_SUGGESTIONS - 1 {
                    self.selected_item += 1;
                }
            }
            let mut draw = self.h.begin_drawing(&self.t);
            draw.clear_background(Color::get_color(0x18181822));
            let mut y_pos = self.ui.pad_top();
            draw.draw_rectangle_rounded(
                Rectangle::new(
                    self.ui.pad_left(),
                    self.ui.pad_top(),
                    640.0 - self.ui.pad_left() * 2.0,
                    self.ui.single_element_h(),
                ),
                0.4,
                12,
                Color::get_color(0xffffff20),
            );
            y_pos += self.ui.char_size().y / 2.0;
            draw.draw_text_ex(
                &self.ui.font,
                &self.buffer,
                Vector2::new(self.ui.pad_left() + self.ui.pad_left() / 2.0, y_pos),
                FONT_SIZE as f32,
                0.0,
                Color::RAYWHITE,
            );
            y_pos += self.ui.char_size().y * 1.5;

            for (i, name) in relevant.iter().enumerate() {
                if i == self.selected_item {
                    draw.draw_rectangle_rounded(
                        Rectangle::new(
                            self.ui.pad_left(),
                            self.ui.single_element_h() * (i + 1) as f32
                                + self.ui.pad_top()
                                + self.ui.margin_y(),
                            640.0 - self.ui.pad_left() * 2.0,
                            self.ui.single_element_h(),
                        ),
                        0.4,
                        12,
                        Color::get_color(0xffffff20),
                    );
                }
                y_pos += self.ui.char_size().y / 2.0;
                draw.draw_text_ex(
                    &self.ui.font,
                    &name,
                    Vector2::new(
                        self.ui.pad_left() + self.ui.pad_left() / 2.0,
                        y_pos + self.ui.margin_y(),
                    ),
                    FONT_SIZE as f32,
                    0.0,
                    Color::RAYWHITE,
                );
                y_pos += self.ui.char_size().y * 1.5;
            }
        }
    }
}

fn main() {
    let r = RMenu::new();
    r.run();
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
