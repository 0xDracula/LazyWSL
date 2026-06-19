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

pub fn next_snapshot_path(distro: &str) -> std::io::Result<PathBuf> {
    let dir = snapshot_distros_dir().join(distro);
    fs::create_dir_all(&dir)?;

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut max_n: u32 = 0;
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name();
            let name = name.to_string_lossy();

            if let Some(rest) = name.strip_prefix(&format!("{date}_")) {
                if let Some(n_str) = rest.strip_suffix(".tar") {
                    if let Ok(n) = n_str.parse::<u32>() {
                        max_n = max_n.max(n);
                    }
                }
            }
        }
    }

    let next = max_n + 1;
    Ok(dir.join(format!("{date}_{:03}.tar", next)))
}

#[cfg(test)]
mod tests {
    use std::fs;

    fn max_index_in(dir: &std::path::Path, date: &str) -> u32 {
        let mut max_n = 0u32;
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let name = e.file_name();
                let name = name.to_string_lossy();
                if let Some(rest) = name.strip_prefix(&format!("{date}_")) {
                    if let Some(n_str) = rest.strip_suffix(".tar") {
                        if let Ok(n) = n_str.parse::<u32>() {
                            max_n = max_n.max(n);
                        }
                    }
                }
            }
        }
        max_n
    }

    #[test]
    fn finds_highest_same_day_index() {
        let tmp = std::env::temp_dir().join(format!("lazywsl_snap_{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();
        let date = "2026-06-19";
        fs::write(tmp.join(format!("{date}_001.tar")), b"x").unwrap();
        fs::write(tmp.join(format!("{date}_002.tar")), b"x").unwrap();
        fs::write(tmp.join(format!("{date}_003.tar")), b"x").unwrap();

        assert_eq!(
            max_index_in(&tmp, date),
            3,
            "next would be _004, not overwrite _001"
        );

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn empty_dir_yields_zero() {
        let tmp = std::env::temp_dir().join(format!("lazywsl_snap_empty_{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();
        assert_eq!(max_index_in(&tmp, "2026-06-19"), 0);
        fs::remove_dir_all(&tmp).ok();
    }
}
