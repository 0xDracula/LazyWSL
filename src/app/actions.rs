use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::KeymapConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    Help,
    HealthCheckPrompt,
    RunSelected,
    OpenShell,
    Terminate,
    SetDefault,
    UnregisterPrompt,
    ShutdownPrompt,
    ExportPrompt,
    ImportPrompt,
    MoveSelection(isize),
    Ignore,
    CustomActionsPrompt,
    SearchPrompt,
    ClearSearch,
    TogglePin,
    ToggleMultiSelect,
    ClonePrompt,
    RollBackPrompt,
    SnapshotPrompt,
    SnapshotManagerPrompt,
    OpenCatalogPrompt,
}

fn normalize_key(value: &str) -> String {
    let value = value.trim().replace(' ', "");
    if value.chars().count() == 1 {
        value
    } else {
        value.to_lowercase()
    }
}

fn plain_key_name(code: KeyCode) -> Option<String> {
    match code {
        KeyCode::Backspace => Some("backspace".to_string()),
        KeyCode::Enter => Some("enter".to_string()),
        KeyCode::Left => Some("left".to_string()),
        KeyCode::Right => Some("right".to_string()),
        KeyCode::Up => Some("up".to_string()),
        KeyCode::Down => Some("down".to_string()),
        KeyCode::End => Some("end".to_string()),
        KeyCode::PageUp => Some("pageup".to_string()),
        KeyCode::PageDown => Some("pagedown".to_string()),
        KeyCode::Tab => Some("tab".to_string()),
        KeyCode::BackTab => Some("shift+tab".to_string()),
        KeyCode::Delete => Some("delete".to_string()),
        KeyCode::Insert => Some("insert".to_string()),
        KeyCode::Esc => Some("esc".to_string()),
        KeyCode::Char(' ') => Some("space".to_string()),
        KeyCode::Char(c) => Some(c.to_string()),
        KeyCode::F(n) => Some(format!("f{n}")),
        _ => None,
    }
}

fn key_candidates(key: KeyEvent) -> Vec<String> {
    let Some(plain) = plain_key_name(key.code) else {
        return Vec::new();
    };

    let mut modifiers = Vec::new();

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        modifiers.push("ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        modifiers.push("alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        modifiers.push("shift");
    }

    let mut candidates = if modifiers.is_empty() {
        vec![normalize_key(&plain)]
    } else {
        Vec::new()
    };

    if !modifiers.is_empty() {
        candidates.push(format!("{}+{}", modifiers.join("+"), normalize_key(&plain)));
    }

    if let KeyCode::Char(c) = key.code
        && c.is_ascii_uppercase()
    {
        candidates.push(c.to_string());
        candidates.push(format!("shift+{}", c.to_ascii_lowercase()));
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

fn matches_any(key: KeyEvent, bindings: &[String]) -> bool {
    let candidates = key_candidates(key);
    bindings
        .iter()
        .map(|binding| normalize_key(binding))
        .any(|binding| candidates.iter().any(|candidate| candidate == &binding))
}

pub fn map_key(key: KeyEvent, keymaps: &KeymapConfig) -> AppAction {
    if matches_any(key, &keymaps.quit) {
        AppAction::Quit
    } else if matches_any(key, &keymaps.help) {
        AppAction::Help
    } else if matches_any(key, &keymaps.health) {
        AppAction::HealthCheckPrompt
    } else if matches_any(key, &keymaps.run) {
        AppAction::RunSelected
    } else if matches_any(key, &keymaps.open_shell) {
        AppAction::OpenShell
    } else if matches_any(key, &keymaps.terminate) {
        AppAction::Terminate
    } else if matches_any(key, &keymaps.set_default) {
        AppAction::SetDefault
    } else if matches_any(key, &keymaps.unregister) {
        AppAction::UnregisterPrompt
    } else if matches_any(key, &keymaps.shutdown) {
        AppAction::ShutdownPrompt
    } else if matches_any(key, &keymaps.export) {
        AppAction::ExportPrompt
    } else if matches_any(key, &keymaps.import) {
        AppAction::ImportPrompt
    } else if matches_any(key, &keymaps.custom_actions) {
        AppAction::CustomActionsPrompt
    } else if matches_any(key, &keymaps.search) {
        AppAction::SearchPrompt
    } else if matches_any(key, &keymaps.clear_search) {
        AppAction::ClearSearch
    } else if matches_any(key, &keymaps.toggle_pin) {
        AppAction::TogglePin
    } else if matches_any(key, &keymaps.toggle_multi_select) {
        AppAction::ToggleMultiSelect
    } else if matches_any(key, &keymaps.move_down) {
        AppAction::MoveSelection(1)
    } else if matches_any(key, &keymaps.move_up) {
        AppAction::MoveSelection(-1)
    } else if matches_any(key, &keymaps.clone) {
        AppAction::ClonePrompt
    } else if matches_any(key, &keymaps.rollback) {
        AppAction::RollBackPrompt
    } else if matches_any(key, &keymaps.snapshot) {
        AppAction::SnapshotPrompt
    } else if matches_any(key, &keymaps.snapshot_manager) {
        AppAction::SnapshotManagerPrompt
    } else if matches_any(key, &keymaps.catalog) {
        AppAction::OpenCatalogPrompt
    } else {
        AppAction::Ignore
    }
}

pub fn display_keys(bindings: &[String]) -> String {
    bindings
        .iter()
        .map(|binding| binding.trim())
        .filter(|binding| !binding.is_empty())
        .map(|binding| match normalize_key(binding).as_str() {
            "enter" => "⏎".to_string(),
            "space" => "Space".to_string(),
            "esc" => "Esc".to_string(),
            "up" => "↑".to_string(),
            "down" => "↓".to_string(),
            "left" => "←".to_string(),
            "right" => "→".to_string(),
            _ => binding.to_string(),
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn defaults_match_existing_navigation_keys() {
        let keymaps = KeymapConfig::default();

        assert_eq!(
            map_key(key(KeyCode::Char('H')), &keymaps),
            AppAction::HealthCheckPrompt
        );
        assert_eq!(map_key(key(KeyCode::Enter), &keymaps), AppAction::OpenShell);
        assert_eq!(
            map_key(key(KeyCode::Down), &keymaps),
            AppAction::MoveSelection(1)
        );
    }

    #[test]
    fn custom_bindings_override_unbound_keys() {
        let mut keymaps = KeymapConfig::default();
        keymaps.health = vec!["ctrl+h".to_string()];

        let ctrl_h = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL);

        assert_eq!(map_key(ctrl_h, &keymaps), AppAction::HealthCheckPrompt);
        assert_eq!(
            map_key(key(KeyCode::Char('H')), &keymaps),
            AppAction::Ignore
        );
    }
}
