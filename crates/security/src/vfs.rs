use crate::capability::Permission;
use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("path escapes sandbox boundary: {0}")]
    SandboxEscape(PathBuf),
    #[error("path not allowed by capability: {0}")]
    NotAllowed(PathBuf),
    #[error("mount point not found: {0}")]
    MountNotFound(String),
    #[error("symlink traversal detected: {0}")]
    SymlinkTraversal(PathBuf),
    #[error("no sandbox root configured for skill: {0}")]
    NoSandbox(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("copy-on-write staging error: {0}")]
    CowError(String),
    #[error("path canonicalization failed: {0}")]
    Canonicalize(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;

/// A logical VFS entry that can be either a sandboxed temp dir or a read-only
/// host mount.
#[derive(Debug, Clone)]
pub enum VfsEntry {
    /// A per-skill temporary directory that the skill owns completely.
    SandboxPath(PathBuf),
    /// A host path exposed read-only into the VFS.
    HostMount(PathBuf),
}

/// Validates paths against the capability token's `FsRead` / `FsWrite`
/// permissions without requiring the full token—just the extracted lists.
#[derive(Debug, Default)]
pub struct AllowlistChecker {
    read_roots: Vec<PathBuf>,
    write_roots: Vec<PathBuf>,
}

impl AllowlistChecker {
    pub fn new(permissions: &[Permission]) -> Self {
        let mut checker = Self::default();
        for perm in permissions {
            match perm {
                Permission::FsRead(p) => checker.read_roots.push(p.clone()),
                Permission::FsWrite(p) => {
                    checker.write_roots.push(p.clone());
                    checker.read_roots.push(p.clone());
                }
                _ => {}
            }
        }
        checker
    }

    pub fn check_read(&self, path: &Path) -> Result<()> {
        if self
            .read_roots
            .iter()
            .any(|root| root == &PathBuf::from("*") || path.starts_with(root))
        {
            Ok(())
        } else {
            Err(Error::NotAllowed(path.to_path_buf()))
        }
    }

    pub fn check_write(&self, path: &Path) -> Result<()> {
        if self
            .write_roots
            .iter()
            .any(|root| root == &PathBuf::from("*") || path.starts_with(root))
        {
            Ok(())
        } else {
            Err(Error::NotAllowed(path.to_path_buf()))
        }
    }
}

/// Routes a logical skill-relative path to a concrete host path.
#[derive(Debug, Default)]
pub struct VfsRouter {
    /// skill_id → VfsEntry
    mounts: HashMap<String, Vec<VfsEntry>>,
}

impl VfsRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_mount(&mut self, skill_id: impl Into<String>, entry: VfsEntry) {
        self.mounts
            .entry(skill_id.into())
            .or_default()
            .push(entry);
    }

    pub fn mounts_for(&self, skill_id: &str) -> &[VfsEntry] {
        self.mounts.get(skill_id).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// Staging layer for copy-on-write semantics: accumulates writes that can be
/// committed atomically or rolled back cleanly.
#[derive(Debug, Default)]
pub struct CopyOnWriteLayer {
    staged: HashMap<PathBuf, Vec<u8>>,
    deletes: Vec<PathBuf>,
}

impl CopyOnWriteLayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stages a write to `path`.
    pub fn stage_write(&mut self, path: PathBuf, content: Vec<u8>) {
        self.staged.insert(path, content);
    }

    /// Stages a delete for `path`.
    pub fn stage_delete(&mut self, path: PathBuf) {
        self.staged.remove(&path);
        self.deletes.push(path);
    }

    /// Returns the staged content for `path` if it exists in the staging area.
    pub fn get_staged(&self, path: &Path) -> Option<&Vec<u8>> {
        self.staged.get(path)
    }

    /// Commits all staged writes and deletes to the real filesystem.
    pub fn commit(&mut self) -> Result<()> {
        for path in self.deletes.drain(..) {
            if path.exists() {
                fs::remove_file(&path)
                    .map_err(|_| Error::CowError(format!("delete failed: {}", path.display())))?;
            }
        }
        for (path, content) in self.staged.drain() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, &content)?;
        }
        Ok(())
    }

    /// Discards all staged changes without touching the filesystem.
    pub fn rollback(&mut self) {
        self.staged.clear();
        self.deletes.clear();
    }

    /// Returns true if there are no pending changes.
    pub fn is_clean(&self) -> bool {
        self.staged.is_empty() && self.deletes.is_empty()
    }
}

/// The primary VFS overlay that combines routing, allowlist checking,
/// symlink protection, and copy-on-write staging.
pub struct VfsOverlay {
    #[allow(dead_code)]
    router: VfsRouter,
    /// sandbox root per skill id
    sandbox_roots: HashMap<String, PathBuf>,
    cow: CopyOnWriteLayer,
}

impl VfsOverlay {
    pub fn new(router: VfsRouter) -> Self {
        Self {
            router,
            sandbox_roots: HashMap::new(),
            cow: CopyOnWriteLayer::new(),
        }
    }

    /// Registers or overrides the sandbox root for a skill. The directory is
    /// created if it does not exist.
    pub fn set_sandbox_root(
        &mut self,
        skill_id: impl Into<String>,
        root: PathBuf,
    ) -> Result<()> {
        fs::create_dir_all(&root)?;
        self.sandbox_roots.insert(skill_id.into(), root);
        Ok(())
    }

    /// Resolves a skill-relative path to an absolute host path, performing
    /// symlink traversal protection.
    pub fn resolve_path(&self, skill_id: &str, rel: &Path) -> Result<PathBuf> {
        let root = self
            .sandbox_roots
            .get(skill_id)
            .ok_or_else(|| Error::NoSandbox(skill_id.to_owned()))?;

        let joined = root.join(strip_leading_slash(rel));
        let canonical = self.safe_canonicalize(&joined, root)?;
        Ok(canonical)
    }

    /// Checks whether a resolved absolute path is allowed for reading based on
    /// the provided checker.
    pub fn check_read(&self, checker: &AllowlistChecker, path: &Path) -> Result<()> {
        checker.check_read(path)
    }

    /// Checks whether a resolved absolute path is allowed for writing.
    pub fn check_write(&self, checker: &AllowlistChecker, path: &Path) -> Result<()> {
        checker.check_write(path)
    }

    /// Stages a write into the CoW layer (does not touch disk yet).
    pub fn write_staged(
        &mut self,
        checker: &AllowlistChecker,
        path: PathBuf,
        content: Vec<u8>,
    ) -> Result<()> {
        self.check_write(checker, &path)?;
        self.cow.stage_write(path, content);
        Ok(())
    }

    /// Commits all staged writes to disk.
    pub fn commit(&mut self) -> Result<()> {
        self.cow.commit()
    }

    /// Rolls back all staged writes.
    pub fn rollback(&mut self) {
        self.cow.rollback();
    }

    /// Reads a file, preferring staged content over the real filesystem.
    pub fn read(&self, checker: &AllowlistChecker, path: &Path) -> Result<Vec<u8>> {
        self.check_read(checker, path)?;
        if let Some(staged) = self.cow.get_staged(path) {
            return Ok(staged.clone());
        }
        Ok(fs::read(path)?)
    }

    /// Resolves a path and ensures it stays within `root` even after following
    /// symlinks. Uses a component-by-component walk that respects the boundary.
    fn safe_canonicalize(&self, path: &Path, root: &Path) -> Result<PathBuf> {
        let mut current = PathBuf::new();
        let components: Vec<Component> = path.components().collect();

        for component in components {
            match component {
                Component::Normal(name) => current.push(name),
                Component::RootDir => current.push("/"),
                Component::Prefix(p) => current.push(p.as_os_str()),
                Component::CurDir => {}
                Component::ParentDir => {
                    current.pop();
                }
            }

            // If the current accumulated path is a symlink, resolve it and
            // verify the target remains within root.
            if current.is_symlink() {
                let target = fs::read_link(&current).map_err(|_| Error::SymlinkTraversal(current.clone()))?;
                let resolved = if target.is_absolute() {
                    target
                } else {
                    current
                        .parent()
                        .unwrap_or(Path::new("/"))
                        .join(target)
                };
                if !resolved.starts_with(root) {
                    return Err(Error::SymlinkTraversal(resolved));
                }
                current = resolved;
            }
        }

        if !current.starts_with(root) {
            return Err(Error::SandboxEscape(current));
        }
        Ok(current)
    }
}

fn strip_leading_slash(p: &Path) -> &Path {
    p.strip_prefix("/").unwrap_or(p)
}

