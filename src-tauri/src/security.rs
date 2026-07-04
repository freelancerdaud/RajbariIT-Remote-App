// ============================================
// Feature 4: Security PIN — security.rs
// ============================================
// Generates and validates 6-digit PINs for session authentication.

use rand::Rng;
use std::sync::Mutex;
use std::time::{Instant, Duration};

/// Stores the current active PIN and its creation time.
struct PinState {
    pin: String,
    created_at: Instant,
}

static CURRENT_PIN: Mutex<Option<PinState>> = Mutex::new(None);

/// PIN validity duration: 10 minutes
const PIN_VALIDITY: Duration = Duration::from_secs(600);

/// Generate a new random 6-digit PIN.
#[tauri::command]
pub fn generate_pin() -> Result<String, String> {
    let mut rng = rand::thread_rng();
    let pin: u32 = rng.gen_range(100_000..999_999);
    let pin_str = pin.to_string();

    let mut state = CURRENT_PIN.lock().map_err(|e| e.to_string())?;
    *state = Some(PinState {
        pin: pin_str.clone(),
        created_at: Instant::now(),
    });

    Ok(pin_str)
}

/// Get the current active PIN (auto-regenerates if expired).
#[tauri::command]
pub fn get_current_pin() -> Result<String, String> {
    let mut state = CURRENT_PIN.lock().map_err(|e| e.to_string())?;

    match &*state {
        Some(ps) if ps.created_at.elapsed() < PIN_VALIDITY => {
            Ok(ps.pin.clone())
        }
        _ => {
            // PIN expired or not set — generate new one
            let mut rng = rand::thread_rng();
            let pin: u32 = rng.gen_range(100_000..999_999);
            let pin_str = pin.to_string();
            *state = Some(PinState {
                pin: pin_str.clone(),
                created_at: Instant::now(),
            });
            Ok(pin_str)
        }
    }
}

/// Validate a PIN entered by a remote client.
#[tauri::command]
pub fn validate_pin(pin: String) -> Result<bool, String> {
    let state = CURRENT_PIN.lock().map_err(|e| e.to_string())?;

    match &*state {
        Some(ps) if ps.created_at.elapsed() < PIN_VALIDITY => {
            Ok(ps.pin == pin)
        }
        _ => {
            // PIN expired
            Ok(false)
        }
    }
}
