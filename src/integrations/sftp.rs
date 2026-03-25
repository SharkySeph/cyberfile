use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};

/// Connection state for a remote SFTP session
pub struct SftpConnection {
    session: Session,
    host: String,
    user: String,
}

/// A remote file entry, mirroring the local FileEntry concept
#[derive(Debug, Clone)]
pub struct RemoteEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    #[allow(dead_code)]
    pub permissions: u32,
}

impl SftpConnection {
    /// Connect to a remote host via SSH with key-based auth
    pub fn connect(host: &str, port: u16, user: &str) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr)
            .map_err(|e| format!("TCP connection to {} failed: {}", addr, e))?;
        tcp.set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .ok();

        let mut session = Session::new()
            .map_err(|e| format!("Failed to create SSH session: {}", e))?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| format!("SSH handshake failed: {}", e))?;

        // Try SSH agent first
        if session.userauth_agent(user).is_ok() && session.authenticated() {
            return Ok(SftpConnection {
                session,
                host: host.to_string(),
                user: user.to_string(),
            });
        }

        // Try default private key (~/.ssh/id_rsa, id_ed25519)
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let key_files = ["id_ed25519", "id_rsa", "id_ecdsa"];
        for key_name in &key_files {
            let key_path = home.join(".ssh").join(key_name);
            if key_path.exists() {
                if session
                    .userauth_pubkey_file(user, None, &key_path, None)
                    .is_ok()
                    && session.authenticated()
                {
                    return Ok(SftpConnection {
                        session,
                        host: host.to_string(),
                        user: user.to_string(),
                    });
                }
            }
        }

        Err(format!(
            "Authentication failed for {}@{} — no valid key found",
            user, host
        ))
    }

    /// Connect with password authentication
    pub fn connect_with_password(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
    ) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);
        let tcp = TcpStream::connect(&addr)
            .map_err(|e| format!("TCP connection to {} failed: {}", addr, e))?;

        let mut session = Session::new()
            .map_err(|e| format!("Failed to create SSH session: {}", e))?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| format!("SSH handshake failed: {}", e))?;

        session
            .userauth_password(user, password)
            .map_err(|e| format!("Password auth failed: {}", e))?;

        if !session.authenticated() {
            return Err("Authentication failed".to_string());
        }

        Ok(SftpConnection {
            session,
            host: host.to_string(),
            user: user.to_string(),
        })
    }

    /// List directory contents on the remote host
    pub fn list_directory(&self, path: &str) -> Result<Vec<RemoteEntry>, String> {
        let sftp = self
            .session
            .sftp()
            .map_err(|e| format!("SFTP subsystem error: {}", e))?;

        let dir = sftp
            .readdir(Path::new(path))
            .map_err(|e| format!("Failed to read remote directory {}: {}", path, e))?;

        let mut entries = Vec::new();
        for (entry_path, stat) in dir {
            let name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if name == "." || name == ".." {
                continue;
            }

            entries.push(RemoteEntry {
                name,
                path: entry_path.to_string_lossy().to_string(),
                is_dir: stat.is_dir(),
                size: stat.size.unwrap_or(0),
                permissions: stat.perm.unwrap_or(0) as u32,
            });
        }

        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                return b.is_dir.cmp(&a.is_dir);
            }
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        });

        Ok(entries)
    }

    /// Read a remote file's contents into a byte buffer
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        let sftp = self
            .session
            .sftp()
            .map_err(|e| format!("SFTP subsystem error: {}", e))?;

        let mut file = sftp
            .open(Path::new(path))
            .map_err(|e| format!("Failed to open remote file {}: {}", path, e))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| format!("Failed to read remote file: {}", e))?;

        Ok(contents)
    }

    /// Download a remote file to a local destination
    pub fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<(), String> {
        let contents = self.read_file(remote_path)?;
        std::fs::write(local_path, &contents)
            .map_err(|e| format!("Failed to write local file: {}", e))
    }

    /// Upload a local file to the remote host
    #[allow(dead_code)]
    pub fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<(), String> {
        let contents = std::fs::read(local_path)
            .map_err(|e| format!("Failed to read local file: {}", e))?;

        let sftp = self
            .session
            .sftp()
            .map_err(|e| format!("SFTP subsystem error: {}", e))?;

        let mut file = sftp
            .create(Path::new(remote_path))
            .map_err(|e| format!("Failed to create remote file: {}", e))?;

        use std::io::Write;
        file.write_all(&contents)
            .map_err(|e| format!("Failed to write remote file: {}", e))?;

        Ok(())
    }

    /// Get display name for the connection
    pub fn display_name(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }
}

/// Parse an sftp:// URI into (host, port, user, path) components
/// Format: sftp://user@host:port/path or sftp://user@host/path
#[allow(dead_code)]
pub fn parse_sftp_uri(uri: &str) -> Option<(String, u16, String, String)> {
    let stripped = uri.strip_prefix("sftp://")?;

    let (user_host, path) = if let Some(idx) = stripped.find('/') {
        (&stripped[..idx], &stripped[idx..])
    } else {
        (stripped, "/")
    };

    let (user, host_port) = if let Some(idx) = user_host.find('@') {
        (&user_host[..idx], &user_host[idx + 1..])
    } else {
        let current_user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        return Some((
            host_port_split(user_host).0,
            host_port_split(user_host).1,
            current_user,
            path.to_string(),
        ));
    };

    let (host, port) = host_port_split(host_port);
    Some((host, port, user.to_string(), path.to_string()))
}

#[allow(dead_code)]
fn host_port_split(s: &str) -> (String, u16) {
    if let Some(idx) = s.find(':') {
        let host = s[..idx].to_string();
        let port = s[idx + 1..].parse().unwrap_or(22);
        (host, port)
    } else {
        (s.to_string(), 22)
    }
}
