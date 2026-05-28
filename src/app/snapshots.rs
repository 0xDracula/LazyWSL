use crate::config;
use std::{fs, path::PathBuf};

pub fn snapshot_distros_dir() -> PathBuf {
    config::config_dir().join("snapshots").join("distros")
}

pub fn list_snapshot_distros() -> Vec<String> {
    let root = snapshot_distros_dir();
    let mut out = vec![];

    if let Ok(rd) = fs::read_dir(root) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    out.push(name.to_string());
                }
            }
        }
    }

    out.sort();
    out
}

pub fn list_snapshots_for_distro(distro: &str) -> Vec<PathBuf> {
    let dir = snapshot_distros_dir().join(distro);
    let mut out = vec![];

    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "tar") {
                out.push(p);
            }
        }
    }

    out.sort();
    out.reverse();
    out
}
