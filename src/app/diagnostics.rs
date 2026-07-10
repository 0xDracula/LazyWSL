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

pub fn format_distro_list(names: &[String]) -> String {
    const MAX_VISIBLE: usize = 4;

    if names.len() <= MAX_VISIBLE {
        names.join(", ")
    } else {
        format!(
            "{}, +{} more",
            names[..MAX_VISIBLE].join(", "),
            names.len() - MAX_VISIBLE
        )
    }
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

        let wsl1_distros = distributions
            .iter()
            .filter(|d| d.version == WslVersion::V1)
            .map(|d| d.name.clone())
            .collect::<Vec<_>>();

        let unknown_state_distros = distributions
            .iter()
            .filter(|d| matches!(d.state, DistroState::Unknown(_)))
            .map(|d| format!("{} ({})", d.name, d.state))
            .collect::<Vec<_>>();

        let unknown_version_distros = distributions
            .iter()
            .filter(|d| matches!(d.version, WslVersion::Unknown(_)))
            .map(|d| format!("{} (WSL {})", d.name, d.version))
            .collect::<Vec<_>>();

        let missing_install_path_distros = distributions
            .iter()
            .filter(|d| d.install_path.is_none())
            .map(|d| d.name.clone())
            .collect::<Vec<_>>();

        let oversized_distros = distributions
            .iter()
            .filter_map(|d| {
                let size = d.size_bytes?;
                (size > 20 * 1024 * 1024 * 1024)
                    .then(|| format!("{} ({})", d.name, snapshots::format_size(size)))
            })
            .collect::<Vec<_>>();

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

        if wsl1_distros.is_empty() {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "WSL version".to_string(),
                detail: format!(
                    "{} distro(s) are still on WSL 1: {}",
                    wsl1_distros.len(),
                    format_distro_list(&wsl1_distros)
                ),
            });
        } else if distro_count > 0 && unknown_version_distros.is_empty() {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Ok,
                label: "WSL version".to_string(),
                detail: "All detected distros use WSL 2".to_string(),
            });
        }

        if !unknown_state_distros.is_empty() {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Unknown states".to_string(),
                detail: format!(
                    "{} distro(s) reported an unkown state: {}",
                    unknown_state_distros.len(),
                    format_distro_list(&unknown_state_distros)
                ),
            });
        }

        if !unknown_version_distros.is_empty() {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Unkown versions".to_string(),
                detail: format!(
                    "{} distro(s) reported an unkown WSL version: {}",
                    unknown_version_distros.len(),
                    format_distro_list(&unknown_version_distros)
                ),
            });
        }

        if !missing_install_path_distros.is_empty() {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Install paths".to_string(),
                detail: format!(
                    "{} distro(s) are missing install path metadata: {}",
                    missing_install_path_distros.len(),
                    format_distro_list(&missing_install_path_distros)
                ),
            });
        }

        if !oversized_distros.is_empty() {
            items.push(DiagnosticItem {
                level: DiagnosticLevel::Warning,
                label: "Distro storage".to_string(),
                detail: format!(
                    "{} distro(s) are larger than 20 GiB: {}",
                    oversized_distros.len(),
                    format_distro_list(&oversized_distros)
                ),
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

        assert!(report.items.iter().any(|item| item.label == "WSL version"
            && item.level == DiagnosticLevel::Warning
            && item.detail.contains("Legacy")));
    }

    #[test]
    fn reports_unknown_state_with_distro_name() {
        let report = DiagnosticReport::from_state(&[distro(
            "BrokenState",
            DistroState::Unknown("yay".to_string()),
            WslVersion::V2,
            true,
        )]);

        assert!(report.items.iter().any(|item| {
            item.label == "Unknown states"
                && item.level == DiagnosticLevel::Warning
                && item.detail.contains("BrokenState")
                && item.detail.contains("yay")
        }))
    }

    #[test]
    fn reports_unknown_version_with_distro_name() {
        let report = DiagnosticReport::from_state(&[distro(
            "FutureWSL",
            DistroState::Stopped,
            WslVersion::Unknown(3),
            true,
        )]);

        assert!(report.items.iter().any(|item| {
            item.label == "Uknown versions"
                && item.level == DiagnosticLevel::Warning
                && item.detail.contains("FutureWSL")
                && item.detail.contains("WSL 3")
        }));

        assert!(
            !report
                .items
                .iter()
                .any(|item| item.detail == "All detected distros use WSL 2")
        );
    }

    #[test]
    fn reports_large_distro_with_name_and_size() {
        let mut large = distro("HugeUbuntu", DistroState::Stopped, WslVersion::V2, true);
        large.install_path = Some("/ws/HugeUbuntu".to_string());
        large.size_bytes = Some(21 * 1024 * 1024 * 1024);

        let report = DiagnosticReport::from_state(&[large]);

        assert!(report.items.iter().any(|item| {
            item.label == "Distro storage"
                && item.level == DiagnosticLevel::Warning
                && item.detail.contains("HugeUbuntu")
        }));
    }
}
