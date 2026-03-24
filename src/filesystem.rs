use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub is_hidden: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub permissions: u32,
}

impl FileEntry {
    pub fn formatted_size(&self) -> String {
        if self.is_dir {
            "—".into()
        } else {
            bytesize::ByteSize(self.size).to_string()
        }
    }

    pub fn formatted_modified(&self) -> String {
        match self.modified {
            Some(time) => {
                let datetime: chrono::DateTime<chrono::Local> = time.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            }
            None => "—".into(),
        }
    }

    pub fn permission_string(&self) -> String {
        let mode = self.permissions;
        let file_type = if self.is_dir {
            'd'
        } else if self.is_symlink {
            'l'
        } else {
            '-'
        };

        let perms = [
            if mode & 0o400 != 0 { 'r' } else { '-' },
            if mode & 0o200 != 0 { 'w' } else { '-' },
            if mode & 0o100 != 0 { 'x' } else { '-' },
            if mode & 0o040 != 0 { 'r' } else { '-' },
            if mode & 0o020 != 0 { 'w' } else { '-' },
            if mode & 0o010 != 0 { 'x' } else { '-' },
            if mode & 0o004 != 0 { 'r' } else { '-' },
            if mode & 0o002 != 0 { 'w' } else { '-' },
            if mode & 0o001 != 0 { 'x' } else { '-' },
        ];

        format!("{}{}", file_type, perms.iter().collect::<String>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortColumn {
    Name,
    Size,
    Modified,
    Permissions,
}

pub fn read_directory(path: &Path, show_hidden: bool) -> Result<Vec<FileEntry>, std::io::Error> {
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let is_symlink = file_type.is_symlink();

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        let is_hidden = name.starts_with('.');

        if !show_hidden && is_hidden {
            continue;
        }

        entries.push(FileEntry {
            name,
            path: entry.path(),
            is_dir: metadata.is_dir(),
            is_symlink,
            is_hidden,
            size: metadata.len(),
            modified: metadata.modified().ok(),
            permissions: metadata.permissions().mode(),
        });
    }

    Ok(entries)
}

pub fn sort_entries(entries: &mut [FileEntry], column: SortColumn, ascending: bool) {
    // Always sort directories first
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let ord = match column {
                    SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    SortColumn::Size => a.size.cmp(&b.size),
                    SortColumn::Modified => a.modified.cmp(&b.modified),
                    SortColumn::Permissions => a.permissions.cmp(&b.permissions),
                };
                if ascending {
                    ord
                } else {
                    ord.reverse()
                }
            }
        }
    });
}

pub fn create_directory(parent: &Path, name: &str) -> Result<PathBuf, std::io::Error> {
    // Reject path traversal in directory names
    if name.contains('/') || name.contains('\0') || name == ".." || name == "." {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid directory name",
        ));
    }
    let path = parent.join(name);
    std::fs::create_dir(&path)?;
    Ok(path)
}

pub fn delete_to_trash(path: &Path) -> Result<(), std::io::Error> {
    // Move to XDG trash directory
    let trash_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Trash")
        .join("files");
    std::fs::create_dir_all(&trash_dir)?;

    let file_name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut dest = trash_dir.join(&file_name);

    // Handle name conflicts
    let mut counter = 1;
    while dest.exists() {
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        dest = trash_dir.join(format!("{}.{}{}", stem, counter, ext));
        counter += 1;
    }

    std::fs::rename(path, dest)
}

pub fn copy_file(src: &Path, dest_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let name = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dest = dest_dir.join(&name);

    if src.is_dir() {
        copy_dir_recursive(src, &dest)?;
    } else {
        std::fs::copy(src, &dest)?;
    }

    Ok(dest)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dest.join(entry.file_name());

        // Skip symlinks to prevent path traversal
        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

pub fn move_file(src: &Path, dest_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let name = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dest = dest_dir.join(&name);

    // Try rename first (fast, same filesystem)
    match std::fs::rename(src, &dest) {
        Ok(()) => Ok(dest),
        Err(_) => {
            // Fall back to copy + delete
            copy_file(src, dest_dir)?;
            if src.is_dir() {
                std::fs::remove_dir_all(src)?;
            } else {
                std::fs::remove_file(src)?;
            }
            Ok(dest)
        }
    }
}
