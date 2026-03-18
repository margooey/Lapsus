use cidre::cg::{EventFlags, EventSrcStateId};

/// Returns true if the given keycode is currently held down, system-wide.
pub fn is_key_held(keycode: u16) -> bool {
    EventSrcStateId::CombinedSession.key_state(keycode)
}

/// Returns the current modifier flags from the combined session.
pub fn current_modifier_flags() -> u64 {
    EventSrcStateId::CombinedSession.flags_state().0
}

/// Returns true if all the specified modifier flags are currently held.
pub fn are_modifiers_held(required_flags: u64) -> bool {
    if required_flags == 0 {
        return true;
    }
    let current = current_modifier_flags();
    (current & required_flags) == required_flags
}

/// Checks if a full keybind combo (modifiers + optional key) is currently held.
pub fn is_combo_held(keycode: Option<u16>, modifiers: u64) -> bool {
    if !are_modifiers_held(modifiers) {
        return false;
    }
    match keycode {
        Some(kc) => is_key_held(kc),
        None => true, // modifier-only combo
    }
}

// CGEventFlags constants for display purposes
const FLAG_SHIFT: u64 = EventFlags::SHIFT.0;
const FLAG_CTRL: u64 = EventFlags::CTRL.0;
const FLAG_ALT: u64 = EventFlags::ALT.0;
const FLAG_CMD: u64 = EventFlags::CMD.0;
const FLAG_FN: u64 = EventFlags::SECONDARY_FN.0;

/// Masks out device-independent modifier flags only (Shift, Ctrl, Alt, Cmd, Fn).
pub fn clean_modifier_flags(raw_flags: u64) -> u64 {
    raw_flags & (FLAG_SHIFT | FLAG_CTRL | FLAG_ALT | FLAG_CMD | FLAG_FN)
}

/// Returns true if the keycode is a modifier key.
pub fn is_modifier_keycode(keycode: u16) -> bool {
    matches!(keycode, 54..=63)
}

/// Builds a display string for a keybind combo, e.g. "Cmd+A" or "Ctrl+Shift".
pub fn combo_display_name(keycode: Option<u16>, modifiers: u64) -> String {
    let mut parts = Vec::new();

    if modifiers & FLAG_CTRL != 0 {
        parts.push("Ctrl");
    }
    if modifiers & FLAG_ALT != 0 {
        parts.push("Option");
    }
    if modifiers & FLAG_SHIFT != 0 {
        parts.push("Shift");
    }
    if modifiers & FLAG_CMD != 0 {
        parts.push("Cmd");
    }
    if modifiers & FLAG_FN != 0 {
        parts.push("Fn");
    }

    if let Some(kc) = keycode {
        parts.push(keycode_name(kc));
    }

    if parts.is_empty() {
        "Click to set".to_string()
    } else {
        parts.join("+")
    }
}

/// Returns a human-readable name for a macOS virtual keycode.
pub fn keycode_name(keycode: u16) -> &'static str {
    match keycode {
        0 => "A",
        1 => "S",
        2 => "D",
        3 => "F",
        4 => "H",
        5 => "G",
        6 => "Z",
        7 => "X",
        8 => "C",
        9 => "V",
        11 => "B",
        12 => "Q",
        13 => "W",
        14 => "E",
        15 => "R",
        16 => "Y",
        17 => "T",
        18 => "1",
        19 => "2",
        20 => "3",
        21 => "4",
        22 => "6",
        23 => "5",
        24 => "=",
        25 => "9",
        26 => "7",
        27 => "-",
        28 => "8",
        29 => "0",
        30 => "]",
        31 => "O",
        32 => "U",
        33 => "[",
        34 => "I",
        35 => "P",
        36 => "Return",
        37 => "L",
        38 => "J",
        39 => "'",
        40 => "K",
        41 => ";",
        42 => "\\",
        43 => ",",
        44 => "/",
        45 => "N",
        46 => "M",
        47 => ".",
        48 => "Tab",
        49 => "Space",
        50 => "`",
        51 => "Delete",
        53 => "Escape",
        54 => "Right Cmd",
        55 => "Left Cmd",
        56 => "Left Shift",
        57 => "Caps Lock",
        58 => "Left Option",
        59 => "Left Ctrl",
        60 => "Right Shift",
        61 => "Right Option",
        62 => "Right Ctrl",
        63 => "Fn",
        96 => "F5",
        97 => "F6",
        98 => "F7",
        99 => "F3",
        100 => "F8",
        101 => "F9",
        103 => "F11",
        105 => "F13",
        107 => "F14",
        109 => "F10",
        111 => "F12",
        113 => "F15",
        118 => "F4",
        120 => "F2",
        122 => "F1",
        123 => "Left",
        124 => "Right",
        125 => "Down",
        126 => "Up",
        _ => "Unknown",
    }
}
