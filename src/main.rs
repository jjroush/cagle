use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{self, Stylize},
    terminal,
};
use serde_json::Value;
use std::io::{self, Write};
use std::path::PathBuf;
use std::{fs, process};

fn main() {
    // Read project-local settings
    let local_path = PathBuf::from(".claude/settings.local.json");
    if !local_path.exists() {
        eprintln!("No .claude/settings.local.json found in current directory.");
        process::exit(1);
    }

    let local_permissions = read_allow_permissions(&local_path);
    if local_permissions.is_empty() {
        eprintln!("No permissions.allow entries in .claude/settings.local.json");
        process::exit(0);
    }

    // Read global settings
    let global_path = dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".claude/settings.json");

    let mut global_value = if global_path.exists() {
        let text = fs::read_to_string(&global_path).expect("Failed to read global settings");
        serde_json::from_str(&text).expect("Failed to parse global settings JSON")
    } else {
        serde_json::json!({})
    };

    let mut global_permissions = read_allow_from_value(&global_value);

    // Interactive TUI
    let mut stdout = io::stdout();
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide).unwrap();

    let mut selected: usize = 0;
    let mut status_msg: Option<String> = None;

    loop {
        // Draw
        execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
        execute!(stdout, cursor::MoveTo(0, 0)).unwrap();

        execute!(
            stdout,
            style::PrintStyledContent("cagle".bold()),
            style::Print(" — promote project permissions to global\n\r\n\r"),
        )
        .unwrap();

        for (i, perm) in local_permissions.iter().enumerate() {
            let is_global = global_permissions.contains(perm);
            let marker = if is_global { "[✓]" } else { "[ ]" };
            let arrow = if i == selected { ">" } else { " " };

            if i == selected {
                execute!(
                    stdout,
                    style::PrintStyledContent(
                        format!(" {} {} {}", arrow, marker, perm).reverse()
                    ),
                )
                .unwrap();
            } else {
                execute!(
                    stdout,
                    style::Print(format!(" {} {} {}", arrow, marker, perm)),
                )
                .unwrap();
            }
            execute!(stdout, style::Print("\n\r")).unwrap();
        }

        // Status line
        execute!(stdout, style::Print("\n\r")).unwrap();
        if let Some(ref msg) = status_msg {
            execute!(
                stdout,
                style::PrintStyledContent(msg.clone().green()),
                style::Print("\n\r"),
            )
            .unwrap();
        }
        execute!(
            stdout,
            style::PrintStyledContent("Enter: apply globally | q: quit".dark_grey()),
        )
        .unwrap();

        stdout.flush().unwrap();

        // Handle input
        if let Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) = event::read()
        {
            match (code, modifiers) {
                (KeyCode::Char('q'), KeyModifiers::NONE)
                | (KeyCode::Char('c'), KeyModifiers::CONTROL)
                | (KeyCode::Esc, _) => break,
                (KeyCode::Up | KeyCode::Char('k'), _) => {
                    if selected > 0 {
                        selected -= 1;
                    }
                    status_msg = None;
                }
                (KeyCode::Down | KeyCode::Char('j'), _) => {
                    if selected + 1 < local_permissions.len() {
                        selected += 1;
                    }
                    status_msg = None;
                }
                (KeyCode::Enter, _) => {
                    let perm = &local_permissions[selected];
                    if global_permissions.contains(perm) {
                        status_msg = Some(format!("\"{}\" is already global", perm));
                    } else {
                        global_permissions.push(perm.clone());
                        write_allow_to_value(&mut global_value, &global_permissions);
                        write_global_settings(&global_path, &global_value);
                        status_msg = Some(format!("Added \"{}\" to global settings", perm));
                    }
                }
                _ => {}
            }
        }
    }

    // Cleanup
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
    terminal::disable_raw_mode().unwrap();
}

fn read_allow_permissions(path: &PathBuf) -> Vec<String> {
    let text = fs::read_to_string(path).expect("Failed to read settings file");
    let value: Value = serde_json::from_str(&text).expect("Failed to parse JSON");
    read_allow_from_value(&value)
}

fn read_allow_from_value(value: &Value) -> Vec<String> {
    value
        .get("permissions")
        .and_then(|p| p.get("allow"))
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn write_allow_to_value(value: &mut Value, permissions: &[String]) {
    let obj = value.as_object_mut().unwrap();
    let perms_obj = obj
        .entry("permissions")
        .or_insert_with(|| serde_json::json!({}));
    let perms_map = perms_obj.as_object_mut().unwrap();
    let allow_arr: Vec<Value> = permissions.iter().map(|s| Value::String(s.clone())).collect();
    perms_map.insert("allow".to_string(), Value::Array(allow_arr));
}

fn write_global_settings(path: &PathBuf, value: &Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create ~/.claude directory");
    }
    let text = serde_json::to_string_pretty(value).expect("Failed to serialize JSON");
    fs::write(path, text + "\n").expect("Failed to write global settings");
}
