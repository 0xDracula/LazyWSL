use crate::{
    app::snapshots,
    core::Distribution,
    wsl::{DistroState, WslVersion},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticItem {
    pub level: DiagnosticLevel,
    pub label: String,
    pub detail: String,
}

pub struct DiagnosticReport {
    pub summary: String,
    pub items: Vec<DiagnosticItem>,
}

impl DiagnosticReport {
    pub fn from_state(distributions: &[Distribution]) -> Self {
        let mut items = Vec::new();
        let distro_count = distributions.len();
        let running_count = distributions
            .iter()
            .filter(|d| d.state == DistroState::Running)
            .count();

        let stopped_count = distributions
            .iter()
            .filter(|d| d.state == DistroState::Stopped)
            .count();

        let default = distributions.iter().find(|d| d.is_default);

        let wsl1_count = distributions
            .iter()
            .filter(|d| d.version == WslVersion::V1)
            .count();

        let unkown_state_count = distributions
            .iter()
            .filter(|d| matches!(d.state, DistroState::Unknown(_)))
            .count();

        let unkown_version_count = distributions
            .iter()
            .filter(|d| matches!(d.version, WslVersion::Unknown(_)))
            .count();

        if distro_count == 0 {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Error,
                label: "Distros".to_string(),
                detail: "No WSL distributions are loaded. Install one or refresh WSL".to_string(),
            });
        } else {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Ok,
                label: "Distros".to_string(),
                detail: format!(
                    "{distro_count} installed ({running_count} running, {stopped_count} stopped)"
                ),
            });
        }

        match default {
            Some(distro) => items.push(DiagnosticItem {
                level: DiagnosticLevel::Ok,
                label: "Default distro".to_string(),
                detail: format!("{} is set as the default distro.", distro.name),
            }),
            None if distro_count > 0 => items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Default distro".to_string(),
                detail: "No default distro detected. Select one and press d".to_string(),
            }),
            None => {}
        }

        if wsl1_count > 0 {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "WSL version".to_string(),
                detail: format!("{wsl1_count} distro(s) are still on WSL 1"),
            });
        } else if distro_count > 0 {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Ok,
                label: "WSL version".to_string(),
                detail: "All detected distros use WSL 2".to_string(),
            });
        }

        if unkown_state_count > 0 {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Unknown states".to_string(),
                detail: format!("{unkown_state_count} distro(s) reported an uknown state"),
            });
        }

        if unkown_version_count > 0 {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Unkown versions".to_string(),
                detail: format!("{unkown_version_count} distro(s) reported an unkown WSL version"),
            });
        }

        let snap_distros = snapshots::list_snapshot_distros();
        let snapshot_count = snap_distros
            .iter()
            .map(|distro| snapshots::list_snapshot_infos(distro).len())
            .sum::<usize>();
        let snapshot_size = snapshots::total_snapshot_size();

        if snapshot_count == 0 {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Snapshots".to_string(),
                detail: "No snapshots found yet. Press z to create a safety snapshot".to_string(),
            });
        } else {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Ok,
                label: "Snapshots".to_string(),
                detail: format!(
                    "{snapshot_count} snapshot(s), using {}.",
                    snapshots::format_size(snapshot_size)
                ),
            });
        }

        if snapshot_size > 10 * 1024 * 1024 * 1024 {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Snapshot storage".to_string(),
                detail: "Snapshots use more than 10 GiB. Open snapshot manager with S to prune"
                    .to_string(),
            });
        }

        let warnings = items
            .iter()
            .filter(|item| item.level == DiagnosticLevel::Warning)
            .count();

        let errors = items
            .iter()
            .filter(|item| item.level == DiagnosticLevel::Error)
            .count();

        let summary = if errors > 0 {
            format!("{errors} error(s), {warnings} warning(s)")
        } else if warnings > 0 {
            format!("{warnings} warning(s)")
        } else {
            "Everything looks healthy!".to_string()
        };

        Self { summary, items }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distro(
        name: &str,
        state: DistroState,
        version: WslVersion,
        is_default: bool,
    ) -> Distribution {
        Distribution {
            id: None,
            name: name.to_string(),
            state,
            version,
            is_default,
            install_path: None,
            size_bytes: None,
        }
    }

    #[test]
    fn reports_missing_distros_as_error() {
        let report = DiagnosticReport::from_state(&[]);

        assert_eq!(report.items[0].level, DiagnosticLevel::Error);
        assert!(report.summary.contains("error"));
    }

    #[test]
    fn warns_when_default_is_missing() {
        let report = DiagnosticReport::from_state(&[distro(
            "Ubuntu",
            DistroState::Stopped,
            WslVersion::V2,
            false,
        )]);

        assert!(
            report.items.iter().any(
                |item| item.label == "Default distro" && item.level == DiagnosticLevel::Warning
            )
        );
    }

    #[test]
    fn warns_for_wsl1_distros() {
        let report = DiagnosticReport::from_state(&[distro(
            "Legacy",
            DistroState::Stopped,
            WslVersion::V1,
            true,
        )]);

        assert!(
            report
                .items
                .iter()
                .any(|item| item.label == "WSL version" && item.level == DiagnosticLevel::Warning)
        );
    }
}
