use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::actions::AppAction, config::KeymapConfig};

pub fn normalize_key(value: &str) -> String {
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
        KeyCode::Home => Some("home".to_string()),
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
    action_bindings(keymaps)
        .into_iter()
        .find_map(|binding| matches_any(key, binding.keys).then_some(binding.action))
        .unwrap_or(AppAction::Ignore)
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

#[derive(Clone, Copy)]
pub struct ActionBinding<'a> {
    pub action: AppAction,
    pub keys: &'a [String],
    pub label: &'static str,
    pub section: &'static str,
}

pub fn action_bindings(keymaps: &KeymapConfig) -> Vec<ActionBinding<'_>> {
    vec![
        ActionBinding {
            action: AppAction::MoveSelection(-1),
            keys: &keymaps.move_up,
            label: "Move up",
            section: "Navigation",
        },
        ActionBinding {
            action: AppAction::MoveSelection(1),
            keys: &keymaps.move_down,
            label: "Move down",
            section: "Navigation",
        },
        ActionBinding {
            action: AppAction::ToggleMultiSelect,
            keys: &keymaps.toggle_multi_select,
            label: "Multi-select",
            section: "Navigation",
        },
        ActionBinding {
            action: AppAction::SearchPrompt,
            keys: &keymaps.search,
            label: "Search",
            section: "Navigation",
        },
        ActionBinding {
            action: AppAction::HealthCheckPrompt,
            keys: &keymaps.health,
            label: "Health check",
            section: "Navigation",
        },
        ActionBinding {
            action: AppAction::Help,
            keys: &keymaps.help,
            label: "Open this help",
            section: "Navigation",
        },
        ActionBinding {
            action: AppAction::OpenShell,
            keys: &keymaps.open_shell,
            label: "Open shell",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::RunSelected,
            keys: &keymaps.run,
            label: "Run distro",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::Terminate,
            keys: &keymaps.terminate,
            label: "Terminate distro",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::SetDefault,
            keys: &keymaps.set_default,
            label: "Set default",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::TogglePin,
            keys: &keymaps.toggle_pin,
            label: "Pin distro",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::ExportPrompt,
            keys: &keymaps.export,
            label: "Export distro",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::ImportPrompt,
            keys: &keymaps.import,
            label: "Import distro",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::CustomActionsPrompt,
            keys: &keymaps.custom_actions,
            label: "Custom actions",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::ClonePrompt,
            keys: &keymaps.clone,
            label: "Clone",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::OpenCatalogPrompt,
            keys: &keymaps.catalog,
            label: "Install from catalog",
            section: "Distro",
        },
        ActionBinding {
            action: AppAction::SnapshotPrompt,
            keys: &keymaps.snapshot,
            label: "Snapshot distro",
            section: "Snapshots",
        },
        ActionBinding {
            action: AppAction::RollBackPrompt,
            keys: &keymaps.rollback,
            label: "Rollback distro",
            section: "Snapshots",
        },
        ActionBinding {
            action: AppAction::SnapshotManagerPrompt,
            keys: &keymaps.snapshot_manager,
            label: "Snapshot manager",
            section: "Snapshots",
        },
        ActionBinding {
            action: AppAction::UnregisterPrompt,
            keys: &keymaps.unregister,
            label: "Unregister distro",
            section: "Danger Zone",
        },
        ActionBinding {
            action: AppAction::ShutdownPrompt,
            keys: &keymaps.shutdown,
            label: "Shutdown all distros",
            section: "Danger Zone",
        },
        ActionBinding {
            action: AppAction::Quit,
            keys: &keymaps.quit,
            label: "Quit",
            section: "Danger Zone",
        },
    ]
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
        assert_eq!(
            map_key(key(KeyCode::Up), &keymaps),
            AppAction::MoveSelection(-1)
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
