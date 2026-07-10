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
            if p.is_dir()
                && let Some(name) = p.file_name().and_then(|s| s.to_str())
            {
                out.push(name.to_string());
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

            if let Some(rest) = name.strip_prefix(&format!("{date}_"))
                && let Some(n_str) = rest.strip_suffix(".tar")
                && let Ok(n) = n_str.parse::<u32>()
            {
                max_n = max_n.max(n);
            }
        }
    }

    let next = max_n + 1;
    Ok(dir.join(format!("{date}_{:03}.tar", next)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotInfo {
    pub path: PathBuf,
    pub file_name: String,
    pub size_bytes: u64,
    pub modified_secs: Option<u64>,
}

pub fn list_snapshot_infos(distro: &str) -> Vec<SnapshotInfo> {
    let dir = snapshot_distros_dir().join(distro);
    let mut out = vec![];

    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "tar") {
                let meta = e.metadata().ok();
                let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified_secs = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs());

                let file_name = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                out.push(SnapshotInfo {
                    path: p,
                    file_name,
                    size_bytes,
                    modified_secs,
                });
            }
        }
    }

    out.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    out
}

pub fn distro_snapshot_size(distro: &str) -> u64 {
    list_snapshot_infos(distro)
        .iter()
        .map(|s| s.size_bytes)
        .sum()
}

pub fn delete_snapshot(path: &std::path::Path) -> std::io::Result<u64> {
    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    fs::remove_file(path)?;
    Ok(size)
}

pub fn total_snapshot_size() -> u64 {
    list_snapshot_distros()
        .iter()
        .flat_map(|d| list_snapshot_infos(d))
        .map(|s| s.size_bytes)
        .sum()
}

pub fn prune_snapshots(distro: &str, keep: usize) -> std::io::Result<(usize, u64)> {
    let infos = list_snapshot_infos(distro);

    let mut deleted = 0usize;
    let mut freed = 0u64;
    for info in infos.into_iter().skip(keep) {
        let sz = delete_snapshot(&info.path)?;
        deleted += 1;
        freed += sz;
    }

    Ok((deleted, freed))
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn max_index_in(dir: &std::path::Path, date: &str) -> u32 {
        let mut max_n = 0u32;
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let name = e.file_name();
                let name = name.to_string_lossy();
                if let Some(rest) = name.strip_prefix(&format!("{date}_"))
                    && let Some(n_str) = rest.strip_suffix(".tar")
                    && let Ok(n) = n_str.parse::<u32>()
                {
                    max_n = max_n.max(n);
                }
            }
        }
        max_n
    }

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lazywsl_{tag}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
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

    #[test]
    fn format_size_units() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1024 B");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(format_size(3 * 1024 * 1024 * 1024), "3.0 GB");
    }

    #[test]
    fn prune_keeps_newest_n() {
        let dir = scratch("prune");

        for i in 1..=5 {
            fs::write(dir.join(format!("2026-06-19_{i:03}.tar")), vec![0u8; 10]).unwrap();
        }

        let mut infos: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "tar"))
            .collect();

        infos.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
        let to_delete: Vec<_> = infos.into_iter().skip(2).collect();
        assert_eq!(to_delete.len(), 3, "keep 2, delete 3");
        for p in &to_delete {
            assert!(delete_snapshot(p).is_ok());
        }
        let remaining = fs::read_dir(&dir).unwrap().flatten().count();
        assert_eq!(remaining, 2);

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_returns_freed_size() {
        let dir = scratch("del");
        let p = dir.join("2026-06-19_001.tar");
        fs::write(&p, vec![0u8; 4096]).unwrap();
        assert_eq!(delete_snapshot(&p).unwrap(), 4096);
        assert!(!p.exists());
        fs::remove_dir_all(&dir).ok();
    }
}
