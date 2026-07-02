#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use korrodium_sdk::io::Key;
use korrodium_sdk::screen::{self, Color};

// Ported from the kernel's native `src/program/editor` — same field names,
// same per-key logic — with `vga::*`/`crate::fs::*` swapped for the
// `korrodium_sdk::screen`/`fs` wasm ABI, and the native poll-loop swapped
// for `read_key()`, which already yields to the executor while blocked.

const COLOR_TEXT: u8 = screen::color(Color::White, Color::Black);
const COLOR_STATUS: u8 = screen::color(Color::Black, Color::LightGray);
const COLOR_TILDE: u8 = screen::color(Color::DarkGray, Color::Black);

/// Files larger than this won't fit in the read buffer; `fs::read_file`
/// returns `None` and the editor opens as a new (empty) file instead.
const MAX_FILE_SIZE: usize = 32 * 1024;

struct Editor {
    lines: Vec<Vec<u8>>,
    cursor_row: usize,
    cursor_col: usize,
    scroll_offset: usize,
    filename: String,
    modified: bool,
    rows: usize,
    cols: usize,
}

impl Editor {
    fn new(filename: &str, rows: usize, cols: usize) -> Self {
        let mut raw = alloc::vec![0u8; MAX_FILE_SIZE];
        let lines = match korrodium_sdk::fs::read_file(filename, &mut raw) {
            Some(data) => {
                let mut lines: Vec<Vec<u8>> = data.split(|&b| b == b'\n').map(|l| l.to_vec()).collect();
                if lines.is_empty() {
                    lines.push(Vec::new());
                }
                lines
            }
            None => alloc::vec![Vec::new()], // new file (or too large to read)
        };

        Editor {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            filename: String::from(filename),
            modified: false,
            rows,
            cols,
        }
    }

    fn editor_rows(&self) -> usize {
        self.rows - 1 // leave the last row for the status bar
    }

    fn status_row(&self) -> usize {
        self.rows - 1
    }

    // Draw everything — called after every keypress.
    fn draw(&self) {
        let editor_rows = self.editor_rows();
        for screen_row in 0..editor_rows {
            let file_row = screen_row + self.scroll_offset;
            screen::clear_row(screen_row, COLOR_TEXT);
            if file_row < self.lines.len() {
                let line = &self.lines[file_row];
                for (col, &byte) in line.iter().enumerate() {
                    if col >= self.cols {
                        break;
                    }
                    screen::write_at(screen_row, col, byte, COLOR_TEXT);
                }
            } else {
                // Empty rows past end of file show a tilde like vim/nano.
                screen::write_at(screen_row, 0, b'~', COLOR_TILDE);
            }
        }

        self.draw_status();

        let screen_row = self.cursor_row - self.scroll_offset;
        screen::move_cursor(screen_row, self.cursor_col);
    }

    fn draw_status(&self) {
        let status_row = self.status_row();
        screen::clear_row(status_row, COLOR_STATUS);
        let modified_str = if self.modified { " [modified]" } else { "" };
        let status = format!(
            " {} {}  Ln {} Col {} | ^S Save  ^Q Quit",
            self.filename,
            modified_str,
            self.cursor_row + 1,
            self.cursor_col + 1,
        );
        screen::write_str_at(status_row, 0, &status, COLOR_STATUS);
    }

    /// Returns `false` to signal the program should quit.
    fn handle_key(&mut self, key: Key) -> bool {
        match key {
            // Ctrl+Q — quit
            Key::Unicode('\x11') => return false,

            // Ctrl+S — save
            Key::Unicode('\x13') => self.save(),

            // Enter
            Key::Unicode('\n') | Key::Unicode('\r') => self.insert_newline(),

            // Backspace
            Key::Unicode('\x08') => self.backspace(),

            // Regular printable character
            Key::Unicode(c) if (c as u32) >= 0x20 && (c as u32) < 0x80 => self.insert_char(c as u8),

            Key::ArrowUp => self.move_up(),
            Key::ArrowDown => self.move_down(),
            Key::ArrowLeft => self.move_left(),
            Key::ArrowRight => self.move_right(),

            Key::Home => self.cursor_col = 0,
            Key::End => self.cursor_col = self.current_line_len(),

            _ => {}
        }

        true
    }

    fn insert_char(&mut self, byte: u8) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        self.lines[row].insert(col, byte);
        self.cursor_col += 1;
        self.modified = true;
    }

    fn backspace(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;

        if col > 0 {
            self.lines[row].remove(col - 1);
            self.cursor_col -= 1;
            self.modified = true;
        } else if row > 0 {
            // At start of line — merge with previous line.
            let current = self.lines.remove(row);
            let prev_len = self.lines[row - 1].len();
            self.lines[row - 1].extend_from_slice(&current);
            self.cursor_row -= 1;
            self.cursor_col = prev_len;
            self.modified = true;
        }
    }

    fn insert_newline(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let rest = self.lines[row].split_off(col);
        self.lines.insert(row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.modified = true;
        self.scroll_if_needed();
    }

    fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.current_line_len());
            if self.cursor_row < self.scroll_offset {
                self.scroll_offset -= 1;
            }
        }
    }

    fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.current_line_len());
            self.scroll_if_needed();
        }
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.current_line_len();
        }
    }

    fn move_right(&mut self) {
        let len = self.current_line_len();
        if self.cursor_col < len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    fn scroll_if_needed(&mut self) {
        let screen_row = self.cursor_row - self.scroll_offset;
        if screen_row >= self.editor_rows() {
            self.scroll_offset += 1;
        }
    }

    fn current_line_len(&self) -> usize {
        self.lines[self.cursor_row].len()
    }

    fn save(&mut self) {
        let mut data: Vec<u8> = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            data.extend_from_slice(line);
            if i + 1 < self.lines.len() {
                data.push(b'\n');
            }
        }
        if korrodium_sdk::fs::write_file(&self.filename, &data) {
            self.modified = false;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let mut args_buf = [0u8; 256];
    let Some(filename) = korrodium_sdk::args(&mut args_buf).next() else {
        korrodium_sdk::print_str("usage: editor <filename>\n");
        korrodium_sdk::exit(1);
    };

    let mut editor = Editor::new(filename, screen::height(), screen::width());

    screen::clear_screen();
    editor.draw();

    loop {
        let key = korrodium_sdk::read_key();
        if !editor.handle_key(key) {
            break;
        }
        editor.draw();
    }

    screen::clear_screen();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
