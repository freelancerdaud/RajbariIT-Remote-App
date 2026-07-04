// ============================================
// Feature 3: Remote Input Control — input.rs
// ============================================
// Desktop: Simulates mouse and keyboard events using enigo crate.
// Android: Stub functions (input simulation not supported).

use serde::Deserialize;

/// Modifiers state for keyboard events
#[derive(Debug, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

// ======================================================
// Desktop Implementation (Windows, macOS, Linux)
// ======================================================

#[cfg(not(target_os = "android"))]
use enigo::{
    Enigo, Settings, Coordinate, Direction, Key, Button, Axis,
    Keyboard, Mouse,
};

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn send_mouse_event(x: i32, y: i32, action: String) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo error: {}", e))?;

    match action.as_str() {
        "move" => {
            let screen_x = (x as f64 / 65535.0 * 1920.0) as i32;
            let screen_y = (y as f64 / 65535.0 * 1080.0) as i32;
            enigo.move_mouse(screen_x, screen_y, Coordinate::Abs)
                .map_err(|e| format!("Mouse move error: {}", e))?;
        }
        "left_click" => {
            enigo.button(Button::Left, Direction::Click)
                .map_err(|e| format!("Click error: {}", e))?;
        }
        "right_click" => {
            enigo.button(Button::Right, Direction::Click)
                .map_err(|e| format!("Click error: {}", e))?;
        }
        "middle_click" => {
            enigo.button(Button::Middle, Direction::Click)
                .map_err(|e| format!("Click error: {}", e))?;
        }
        "scroll_up" => {
            enigo.scroll(3, Axis::Vertical)
                .map_err(|e| format!("Scroll error: {}", e))?;
        }
        "scroll_down" => {
            enigo.scroll(-3, Axis::Vertical)
                .map_err(|e| format!("Scroll error: {}", e))?;
        }
        _ => {
            return Err(format!("Unknown mouse action: {}", action));
        }
    }

    Ok(())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub fn send_key_event(key: String, modifiers: KeyModifiers) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Enigo error: {}", e))?;

    if modifiers.ctrl {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("Key error: {}", e))?;
    }
    if modifiers.alt {
        enigo.key(Key::Alt, Direction::Press).map_err(|e| format!("Key error: {}", e))?;
    }
    if modifiers.shift {
        enigo.key(Key::Shift, Direction::Press).map_err(|e| format!("Key error: {}", e))?;
    }
    if modifiers.meta {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("Key error: {}", e))?;
    }

    let result = match key.as_str() {
        "Enter" => enigo.key(Key::Return, Direction::Click),
        "Backspace" => enigo.key(Key::Backspace, Direction::Click),
        "Tab" => enigo.key(Key::Tab, Direction::Click),
        "Escape" => enigo.key(Key::Escape, Direction::Click),
        "Delete" => enigo.key(Key::Delete, Direction::Click),
        "ArrowUp" => enigo.key(Key::UpArrow, Direction::Click),
        "ArrowDown" => enigo.key(Key::DownArrow, Direction::Click),
        "ArrowLeft" => enigo.key(Key::LeftArrow, Direction::Click),
        "ArrowRight" => enigo.key(Key::RightArrow, Direction::Click),
        "Home" => enigo.key(Key::Home, Direction::Click),
        "End" => enigo.key(Key::End, Direction::Click),
        "PageUp" => enigo.key(Key::PageUp, Direction::Click),
        "PageDown" => enigo.key(Key::PageDown, Direction::Click),
        " " => enigo.key(Key::Space, Direction::Click),
        "F1" => enigo.key(Key::F1, Direction::Click),
        "F2" => enigo.key(Key::F2, Direction::Click),
        "F3" => enigo.key(Key::F3, Direction::Click),
        "F4" => enigo.key(Key::F4, Direction::Click),
        "F5" => enigo.key(Key::F5, Direction::Click),
        "F6" => enigo.key(Key::F6, Direction::Click),
        "F7" => enigo.key(Key::F7, Direction::Click),
        "F8" => enigo.key(Key::F8, Direction::Click),
        "F9" => enigo.key(Key::F9, Direction::Click),
        "F10" => enigo.key(Key::F10, Direction::Click),
        "F11" => enigo.key(Key::F11, Direction::Click),
        "F12" => enigo.key(Key::F12, Direction::Click),
        c if c.len() == 1 => enigo.text(c),
        _ => enigo.text(&key),
    };
    result.map_err(|e| format!("Key input error: {}", e))?;

    if modifiers.meta {
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("Key error: {}", e))?;
    }
    if modifiers.shift {
        enigo.key(Key::Shift, Direction::Release).map_err(|e| format!("Key error: {}", e))?;
    }
    if modifiers.alt {
        enigo.key(Key::Alt, Direction::Release).map_err(|e| format!("Key error: {}", e))?;
    }
    if modifiers.ctrl {
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("Key error: {}", e))?;
    }

    Ok(())
}

// ======================================================
// Android Stub Implementation
// ======================================================

#[cfg(target_os = "android")]
#[tauri::command]
pub fn send_mouse_event(_x: i32, _y: i32, _action: String) -> Result<(), String> {
    Err("Mouse input simulation is not supported on Android".to_string())
}

#[cfg(target_os = "android")]
#[tauri::command]
pub fn send_key_event(_key: String, _modifiers: KeyModifiers) -> Result<(), String> {
    Err("Keyboard input simulation is not supported on Android".to_string())
}
