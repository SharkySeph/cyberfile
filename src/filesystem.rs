use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
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
    Extension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
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
                    SortColumn::Name => natural_cmp(&a.name, &b.name),
                    SortColumn::Size => a.size.cmp(&b.size),
                    SortColumn::Modified => a.modified.cmp(&b.modified),
                    SortColumn::Permissions => a.permissions.cmp(&b.permissions),
                    SortColumn::Extension => {
                        let ext_a = Path::new(&a.name)
                            .extension()
                            .map(|e| e.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        let ext_b = Path::new(&b.name)
                            .extension()
                            .map(|e| e.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        ext_a.cmp(&ext_b).then_with(|| natural_cmp(&a.name, &b.name))
                    }
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

/// Natural number comparison: "file2" < "file10" instead of lexicographic.
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    let mut ai = a.chars().peekable();
    let mut bi = b.chars().peekable();

    loop {
        match (ai.peek(), bi.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    // Compare as numbers
                    let mut an = String::new();
                    while let Some(&c) = ai.peek() {
                        if c.is_ascii_digit() {
                            an.push(c);
                            ai.next();
                        } else {
                            break;
                        }
                    }
                    let mut bn = String::new();
                    while let Some(&c) = bi.peek() {
                        if c.is_ascii_digit() {
                            bn.push(c);
                            bi.next();
                        } else {
                            break;
                        }
                    }
                    let na: u64 = an.parse().unwrap_or(0);
                    let nb: u64 = bn.parse().unwrap_or(0);
                    match na.cmp(&nb) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    ai.next();
                    bi.next();
                    match ac.cmp(&bc) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                }
            }
        }
    }
}

pub fn validate_entry_name(name: &str) -> Result<(), std::io::Error> {
    if name.is_empty() || name.contains('/') || name.contains('\0') || name == ".." || name == "." {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid entry name",
        ));
    }
    Ok(())
}

pub fn create_directory(parent: &Path, name: &str) -> Result<PathBuf, std::io::Error> {
    validate_entry_name(name)?;
    let path = parent.join(name);
    std::fs::create_dir(&path)?;
    Ok(path)
}

pub fn delete_to_trash(path: &Path) -> Result<String, std::io::Error> {
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

    std::fs::rename(path, &dest)?;
    Ok(dest
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string())
}

pub fn copy_file(src: &Path, dest_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let metadata = std::fs::symlink_metadata(src)?;
    let name = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let dest = dest_dir.join(&name);

    if metadata.file_type().is_symlink() {
        copy_symlink(src, &dest)?;
    } else if metadata.is_dir() {
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

        if file_type.is_symlink() {
            copy_symlink(&entry.path(), &dest_path)?;
        } else if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

fn copy_symlink(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    let target = std::fs::read_link(src)?;
    std::os::unix::fs::symlink(target, dest)
}

pub fn create_file(parent: &Path, name: &str) -> Result<PathBuf, std::io::Error> {
    validate_entry_name(name)?;
    let path = parent.join(name);
    std::fs::File::create(&path)?;
    Ok(path)
}

pub fn recreate_entry(path: &Path, kind: EntryKind) -> Result<(), std::io::Error> {
    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if kind == EntryKind::Directory {
        create_directory(parent, &name).map(|_| ())
    } else {
        create_file(parent, &name).map(|_| ())
    }
}

pub fn move_file(src: &Path, dest_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let metadata = std::fs::symlink_metadata(src)?;
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
            if metadata.file_type().is_symlink() {
                std::fs::remove_file(src)?;
            } else if metadata.is_dir() {
                std::fs::remove_dir_all(src)?;
            } else {
                std::fs::remove_file(src)?;
            }
            Ok(dest)
        }
    }
}

/// List contents of the XDG trash directory.
pub fn list_trash() -> Vec<(String, PathBuf)> {
    let trash_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Trash")
        .join("files");

    let mut entries = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&trash_dir) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            entries.push((name, entry.path()));
        }
    }
    entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    entries
}

/// Restore a named item from trash to a target directory.
pub fn restore_from_trash(name: &str, dest_dir: &Path) -> Result<PathBuf, std::io::Error> {
    validate_entry_name(name)?;

    let trash_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Trash")
        .join("files");

    let src = trash_dir.join(name);
    if !src.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Item not found in trash",
        ));
    }

    let dest = dest_dir.join(name);
    std::fs::rename(&src, &dest)?;
    Ok(dest)
}

/// Empty the entire trash directory. Returns count of removed items.
pub fn empty_trash() -> Result<usize, std::io::Error> {
    let trash_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Trash")
        .join("files");

    let mut count = 0;
    if trash_dir.exists() {
        for entry in std::fs::read_dir(&trash_dir)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
            count += 1;
        }
    }
    Ok(count)
}

/// Create a symbolic link at `link_path` pointing to `target`.
pub fn create_symlink(target: &Path, link_path: &Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(target, link_path)
}

/// Search file contents for a pattern using grep or ripgrep.
/// Returns Vec<(file_path, line_number, line_text)>.
pub fn search_content(dir: &Path, query: &str, max_results: usize) -> Vec<(String, u32, String)> {
    // Try ripgrep first, fall back to grep
    let output = std::process::Command::new("rg")
        .args(["--no-heading", "--line-number", "--max-count", "5", "--max-filesize", "1M"])
        .arg("--")
        .arg(query)
        .arg(dir)
        .output()
        .or_else(|_| {
            std::process::Command::new("grep")
                .args(["-rn", "--max-count=5", "--binary-files=without-match"])
                .arg("--")
                .arg(query)
                .arg(dir)
                .output()
        });

    let mut results = Vec::new();
    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines().take(max_results) {
            // Format: path:line_number:text
            let mut parts = line.splitn(3, ':');
            if let (Some(path), Some(num_str), Some(text)) =
                (parts.next(), parts.next(), parts.next())
            {
                if let Ok(num) = num_str.parse::<u32>() {
                    results.push((path.to_string(), num, text.to_string()));
                }
            }
        }
    }
    results
}

/// List contents of a ZIP archive. Returns Vec<(name, size, is_dir)>.
pub fn list_zip_contents(path: &Path) -> Result<Vec<(String, u64, bool)>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            entries.push((
                entry.name().to_string(),
                entry.size(),
                entry.is_dir(),
            ));
        }
    }
    Ok(entries)
}

/// Extract a ZIP archive to `dest_dir`.
pub fn extract_zip(path: &Path, dest_dir: &Path) -> Result<usize, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut count = 0;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(name) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
            continue;
        };

        if name.components().any(|component| {
            matches!(component, Component::RootDir | Component::Prefix(_) | Component::ParentDir)
        }) {
            continue;
        }

        let out_path = dest_dir.join(&name);
        if entry.is_dir() {
            let _ = std::fs::create_dir_all(&out_path);
        } else {
            if let Some(parent) = out_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut outfile) = std::fs::File::create(&out_path) {
                let _ = std::io::copy(&mut entry, &mut outfile);
                count += 1;
            }
        }
    }
    Ok(count)
}

/// Create a ZIP archive from a list of files/directories.
/// Returns the number of entries written.
pub fn create_zip_archive(files: &[PathBuf], output: &Path) -> Result<usize, String> {
    use std::io::{Read, Write};

    let out_file = std::fs::File::create(output).map_err(|e| e.to_string())?;
    let mut zip_writer = zip::ZipWriter::new(out_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut count = 0;

    fn add_path(
        zip_writer: &mut zip::ZipWriter<std::fs::File>,
        path: &Path,
        base: &Path,
        options: zip::write::SimpleFileOptions,
        count: &mut usize,
    ) -> Result<(), String> {
        let rel = path.strip_prefix(base).unwrap_or(path);
        let name = rel.to_string_lossy().to_string();

        if name.contains("..") {
            return Ok(());
        }

        if path.is_dir() {
            let dir_name = if name.ends_with('/') {
                name.clone()
            } else {
                format!("{}/", name)
            };
            zip_writer
                .add_directory(&dir_name, options)
                .map_err(|e| e.to_string())?;
            *count += 1;
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    add_path(zip_writer, &entry.path(), base, options, count)?;
                }
            }
        } else if path.is_file() {
            zip_writer
                .start_file(&name, options)
                .map_err(|e| e.to_string())?;
            let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            zip_writer.write_all(&buf).map_err(|e| e.to_string())?;
            *count += 1;
        }
        Ok(())
    }

    for file in files {
        let base = file.parent().unwrap_or(file);
        add_path(&mut zip_writer, file, base, options, &mut count)?;
    }

    zip_writer.finish().map_err(|e| e.to_string())?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Mutex, OnceLock};

    fn temp_test_dir(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cyberfile-fs-test-{}-{}", label, unique));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn validate_entry_name_rejects_path_components() {
        assert!(validate_entry_name("..").is_err());
        assert!(validate_entry_name("bad/name").is_err());
        assert!(validate_entry_name("\0bad").is_err());
        assert!(validate_entry_name("valid-name.txt").is_ok());
    }

    #[test]
    fn recreate_entry_respects_directory_kind() {
        let root = temp_test_dir("recreate-dir");
        let target = root.join("sector");

        recreate_entry(&target, EntryKind::Directory).unwrap();

        assert!(target.is_dir());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn recreate_entry_respects_file_kind() {
        let root = temp_test_dir("recreate-file");
        let target = root.join("construct.txt");

        recreate_entry(&target, EntryKind::File).unwrap();

        assert!(target.is_file());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn delete_to_trash_returns_actual_collision_name_and_restore_works() {
        let _guard = env_lock().lock().unwrap();
        let xdg_home = temp_test_dir("xdg-home");
        let dir_a = temp_test_dir("trash-a");
        let dir_b = temp_test_dir("trash-b");
        let file_a = dir_a.join("same.txt");
        let file_b = dir_b.join("same.txt");
        std::fs::write(&file_a, "first").unwrap();
        std::fs::write(&file_b, "second").unwrap();

        let old_xdg_home = std::env::var_os("XDG_DATA_HOME");
        unsafe { std::env::set_var("XDG_DATA_HOME", &xdg_home) };

        let trash_a = delete_to_trash(&file_a).unwrap();
        let trash_b = delete_to_trash(&file_b).unwrap();

        assert_eq!(trash_a, "same.txt");
        assert_ne!(trash_a, trash_b);

        let restored_a = restore_from_trash(&trash_a, &dir_a).unwrap();
        let restored_b = restore_from_trash(&trash_b, &dir_b).unwrap();

        assert_eq!(std::fs::read_to_string(restored_a).unwrap(), "first");
        assert_eq!(std::fs::read_to_string(restored_b).unwrap(), "second");

        match old_xdg_home {
            Some(value) => unsafe { std::env::set_var("XDG_DATA_HOME", value) },
            None => unsafe { std::env::remove_var("XDG_DATA_HOME") },
        }

        let _ = std::fs::remove_dir_all(xdg_home);
        let _ = std::fs::remove_dir_all(dir_a);
        let _ = std::fs::remove_dir_all(dir_b);
    }

    #[test]
    fn copy_file_preserves_symlinks() {
        let src_root = temp_test_dir("symlink-src");
        let dst_root = temp_test_dir("symlink-dst");
        let target = src_root.join("target.txt");
        let link = src_root.join("link.txt");

        std::fs::write(&target, "payload").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let copied = copy_file(&link, &dst_root).unwrap();
        let metadata = std::fs::symlink_metadata(&copied).unwrap();

        assert!(metadata.file_type().is_symlink());
        assert_eq!(std::fs::read_link(&copied).unwrap(), target);

        let _ = std::fs::remove_dir_all(src_root);
        let _ = std::fs::remove_dir_all(dst_root);
    }

    #[test]
    fn extract_zip_skips_unsafe_paths() {
        let root = temp_test_dir("zip");
        let archive_path = root.join("archive.zip");
        let extract_dir = root.join("out");
        let escaped_path = std::env::temp_dir().join(format!(
            "cyberfile-zip-escape-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let file = std::fs::File::create(&archive_path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        writer.start_file("safe/file.txt", options).unwrap();
        writer.write_all(b"safe").unwrap();
        writer.start_file("../escape.txt", options).unwrap();
        writer.write_all(b"escape").unwrap();
        writer
            .start_file(escaped_path.to_string_lossy().as_ref(), options)
            .unwrap();
        writer.write_all(b"absolute").unwrap();
        writer.finish().unwrap();

        let extracted = extract_zip(&archive_path, &extract_dir).unwrap();

        assert_eq!(extracted, 1);
        assert_eq!(std::fs::read_to_string(extract_dir.join("safe/file.txt")).unwrap(), "safe");
        assert!(!extract_dir.join("escape.txt").exists());
        assert!(!escaped_path.exists());

        let _ = std::fs::remove_file(escaped_path);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn create_zip_archive_writes_expected_entries() {
        let root = temp_test_dir("zip-create");
        let nested_dir = root.join("nested");
        let nested_file = nested_dir.join("child.txt");
        let top_file = root.join("top.txt");
        let archive_path = root.join("bundle.zip");

        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(&nested_file, "nested").unwrap();
        std::fs::write(&top_file, "top").unwrap();

        let count = create_zip_archive(&[nested_dir.clone(), top_file.clone()], &archive_path).unwrap();
        let entries = list_zip_contents(&archive_path).unwrap();

        assert!(count >= 3);
        assert!(entries.iter().any(|(name, _, is_dir)| name == "nested/" && *is_dir));
        assert!(entries.iter().any(|(name, _, is_dir)| name == "nested/child.txt" && !*is_dir));
        assert!(entries.iter().any(|(name, _, is_dir)| name == "top.txt" && !*is_dir));

        let _ = std::fs::remove_dir_all(root);
    }
}
