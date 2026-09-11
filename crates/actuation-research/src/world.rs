//! Descriptor-relative research Worlds. Candidate paths never escape through
//! parent traversal, symlinks, renamed roots or hardlinked write targets.
use crate::{Error, Result};
use rustix::fs::{self as rfs, AtFlags, Mode, OFlags};
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
const FILE_LIMIT: u64 = 16 * 1024 * 1024;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
fn io(e: impl std::fmt::Display) -> Error {
    Error::new(format!("research World: {e}"))
}
#[derive(Clone, Debug)]
pub struct World {
    root: PathBuf,
    directory: Arc<File>,
}
impl World {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let meta = fs::symlink_metadata(root).map_err(io)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(Error::new("World root must be a real directory"));
        }
        let path = root.canonicalize().map_err(io)?;
        let fd = rfs::open(
            &path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io)?;
        let directory = File::from(fd);
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let anchored = directory.metadata().map_err(io)?;
            if meta.dev() != anchored.dev() || meta.ino() != anchored.ino() {
                return Err(Error::new("World root changed during admission"));
            }
        }
        Ok(Self {
            root: path,
            directory: Arc::new(directory),
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn verify_root(&self) -> Result<()> {
        // An external specimen receives a path, not our fd. Refuse dispatch if
        // that path has stopped naming the anchored World. Not an OS sandbox.
        let m = fs::symlink_metadata(&self.root).map_err(io)?;
        let own = self.directory.metadata().map_err(io)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if m.dev() != own.dev() || m.ino() != own.ino() || m.file_type().is_symlink() {
                return Err(Error::new(
                    "research World path changed before specimen dispatch",
                ));
            }
        }
        Ok(())
    }
    fn parts(relative: &str) -> Result<Vec<String>> {
        if relative.contains('\\') || relative.contains('\0') {
            return Err(Error::new("invalid portable World path"));
        }
        let mut p = Vec::new();
        for c in Path::new(relative).components() {
            match c {
                Component::Normal(n) => p.push(
                    n.to_str()
                        .ok_or_else(|| Error::new("World paths require UTF8"))?
                        .to_owned(),
                ),
                Component::CurDir => {}
                _ => return Err(Error::new("path escapes research World")),
            }
        }
        Ok(p)
    }
    fn directory_at(&self, parts: &[String], create: bool) -> Result<File> {
        let mut current = self.directory.try_clone().map_err(io)?;
        for p in parts {
            if create {
                match rfs::mkdirat(&current, p, Mode::from_raw_mode(0o700)) {
                    Ok(()) => {}
                    Err(rustix::io::Errno::EXIST) => {}
                    Err(e) => return Err(io(e)),
                }
            }
            current = File::from(
                rfs::openat(
                    &current,
                    p,
                    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                    Mode::empty(),
                )
                .map_err(io)?,
            );
        }
        Ok(current)
    }
    /// Reserve a fresh immediate child World through this anchored directory.
    /// No existing directory, symlink, or concurrent reservation is reused.
    pub fn create_child(&self, name: &str) -> Result<Self> {
        self.verify_root()?;
        let parts = Self::parts(name)?;
        if parts.len() != 1 || parts[0] != name {
            return Err(Error::new("child World requires one plain path component"));
        }
        rfs::mkdirat(&*self.directory, name, Mode::from_raw_mode(0o700)).map_err(io)?;
        let directory = self.directory_at(&parts, false)?;
        let world = Self {
            root: self.root.join(name),
            directory: Arc::new(directory),
        };
        world.verify_root()?;
        Ok(world)
    }
    pub fn read(&self, relative: &str) -> Result<Vec<u8>> {
        let p = Self::parts(relative)?;
        let (name, dirs) = p
            .split_last()
            .ok_or_else(|| Error::new("file path required"))?;
        let dir = self.directory_at(dirs, false)?;
        let file = File::from(
            rfs::openat(
                &dir,
                name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(io)?,
        );
        if !file.metadata().map_err(io)?.is_file() {
            return Err(Error::new("World file must be regular"));
        }
        let mut bytes = Vec::new();
        file.take(FILE_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(io)?;
        if bytes.len() as u64 > FILE_LIMIT {
            return Err(Error::new("World file exceeds bound"));
        }
        Ok(bytes)
    }
    pub fn text(&self, relative: &str) -> Result<String> {
        String::from_utf8(self.read(relative)?).map_err(|_| Error::new("World file is not UTF8"))
    }
    /// Returns true only for a newly created file. Immutable collisions leave
    /// the previous bytes untouched; the caller decides semantic dedup/conflict.
    pub fn write(&self, relative: &str, bytes: &[u8], immutable: bool) -> Result<bool> {
        if bytes.len() as u64 > FILE_LIMIT {
            return Err(Error::new("World write exceeds bound"));
        }
        let p = Self::parts(relative)?;
        let (name, dirs) = p
            .split_last()
            .ok_or_else(|| Error::new("file path required"))?;
        let dir = self.directory_at(dirs, true)?;
        if let Ok(stat) = rfs::statat(&dir, name, AtFlags::SYMLINK_NOFOLLOW) {
            if rfs::FileType::from_raw_mode(stat.st_mode) != rfs::FileType::RegularFile {
                return Err(Error::new("refusing non-regular World write target"));
            }
        }
        let tmp = format!(
            ".actuation-write-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let mut file = File::from(
            rfs::openat(
                &dir,
                &tmp,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            )
            .map_err(io)?,
        );
        let result = (|| {
            file.write_all(bytes).map_err(io)?;
            file.sync_all().map_err(io)?;
            let created = if immutable {
                match rfs::linkat(&dir, &tmp, &dir, name, AtFlags::empty()) {
                    Ok(()) => true,
                    Err(rustix::io::Errno::EXIST) => false,
                    Err(e) => return Err(io(e)),
                }
            } else {
                rfs::renameat(&dir, &tmp, &dir, name).map_err(io)?;
                true
            };
            dir.sync_all().map_err(io)?;
            Ok(created)
        })();
        let _ = rfs::unlinkat(&dir, &tmp, AtFlags::empty());
        result
    }
    pub fn list(&self, relative: &str) -> Result<Vec<Value>> {
        let p = Self::parts(relative)?;
        let dir = self.directory_at(&p, false)?;
        let mut entries = Vec::new();
        let iterator = rfs::Dir::read_from(&dir).map_err(io)?;
        for e in iterator {
            let e = e.map_err(io)?;
            let name = e.file_name().to_str().map_err(io)?;
            if name == "." || name == ".." {
                continue;
            }
            if entries.len() >= 10000 {
                return Err(Error::new("World directory exceeds entry bound"));
            }
            let stat = rfs::statat(&dir, name, AtFlags::SYMLINK_NOFOLLOW).map_err(io)?;
            let kind = match rfs::FileType::from_raw_mode(stat.st_mode) {
                rfs::FileType::Directory => "directory",
                rfs::FileType::RegularFile => "file",
                _ => {
                    return Err(Error::new(
                        "World contains a symlink or unsupported file kind",
                    ))
                }
            };
            entries.push(json!({"name":name,"type":kind}));
        }
        entries.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        Ok(entries)
    }
    pub fn snapshot(&self) -> Result<Value> {
        fn walk(
            w: &World,
            path: &str,
            out: &mut Value,
            total: &mut usize,
            depth: usize,
        ) -> Result<()> {
            if depth > 64 {
                return Err(Error::new("World depth exceeds bound"));
            }
            for e in w.list(path)? {
                let name = e["name"].as_str().unwrap();
                let child = if path == "." {
                    name.to_owned()
                } else {
                    format!("{path}/{name}")
                };
                if e["type"] == "directory" {
                    walk(w, &child, out, total, depth + 1)?;
                } else {
                    let text = w.text(&child)?;
                    *total += text.len();
                    if *total > 64 * 1024 * 1024 || out.as_object().unwrap().len() >= 10000 {
                        return Err(Error::new("World snapshot exceeds bound"));
                    }
                    out[child] = json!(text);
                }
            }
            Ok(())
        }
        let mut result = json!({});
        walk(self, ".", &mut result, &mut 0, 0)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_requires_a_real_directory() {
        assert!(World::open("/definitely/not/here").is_err());
        let file = tempfile::tempdir().expect("tempdir");
        let path = file.path().join("file.txt");
        std::fs::write(&path, b"x").unwrap();
        assert!(World::open(&path).is_err());
        let link = file.path().join("link");
        std::os::unix::fs::symlink(file.path(), &link).unwrap();
        assert!(World::open(&link).is_err());
        assert!(World::open(file.path()).is_ok());
    }

    #[test]
    fn child_worlds_are_created_through_the_anchored_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let world = World::open(dir.path()).unwrap();
        let child = world.create_child("trial-1").expect("child");
        assert!(child.root().is_dir());
        assert_eq!(child.root(), world.root().join("trial-1"));
        assert!(world.create_child("a/b").is_err());
        assert!(world.create_child("../escape").is_err());
        assert!(world.create_child("nested\\name").is_err());
        assert!(world.create_child("").is_err());
    }

    #[test]
    fn paths_may_not_escape_or_traverse() {
        let dir = tempfile::tempdir().expect("tempdir");
        let world = World::open(dir.path()).unwrap();
        for p in [
            "../outside",
            "..",
            "/absolute",
            "a/../../b",
            "back\\slash",
            ".hidden/ok/../x",
            "",
        ] {
            // ".hidden/ok/../x" must also be refused: `..` is not a Normal component.
            let refused = world.read(p).is_err();
            assert!(refused, "expected refusal for {p:?}");
        }
        assert!(world.read("sub/inner/file.txt").is_err());
        world.write("sub/inner/file.txt", b"content", true).unwrap();
        assert_eq!(world.read("sub/inner/file.txt").unwrap(), b"content");
        assert_eq!(world.text("sub/inner/file.txt").unwrap(), "content");
    }

    #[test]
    fn immutable_writes_dedupe_and_never_overwrite() {
        let dir = tempfile::tempdir().expect("tempdir");
        let world = World::open(dir.path()).unwrap();
        assert!(world.write("record.json", b"first", true).unwrap());
        assert!(!world.write("record.json", b"second", true).unwrap());
        assert_eq!(world.read("record.json").unwrap(), b"first");
        assert!(world.write("record.json", b"third", false).unwrap());
        assert_eq!(world.read("record.json").unwrap(), b"third");
    }

    #[test]
    fn writes_refuse_non_regular_targets_and_oversized_payloads() {
        let dir = tempfile::tempdir().expect("tempdir");
        let world = World::open(dir.path()).unwrap();
        world.write("data.json", b"{}", true).unwrap();
        let link = dir.path().join("link.json");
        std::os::unix::fs::symlink("data.json", &link).unwrap();
        assert!(world.write("link.json", b"x", true).is_err());
        assert!(world.read("link.json").is_err());
        let oversized = vec![0u8; (FILE_LIMIT + 1) as usize];
        assert!(world.write("big.bin", &oversized, false).is_err());
    }

    #[test]
    fn verify_root_detects_a_swapped_or_removed_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inner = dir.path().join("world");
        std::fs::create_dir(&inner).unwrap();
        let world = World::open(&inner).unwrap();
        assert!(world.verify_root().is_ok());
        let moved = dir.path().join("moved");
        std::fs::rename(&inner, &moved).unwrap();
        std::os::unix::fs::symlink(&moved, &inner).unwrap();
        assert!(world.verify_root().is_err());
        std::fs::remove_file(&inner).unwrap();
        assert!(world.verify_root().is_err());
    }

    #[test]
    fn list_sorts_and_refuses_symlinks() {
        let dir = tempfile::tempdir().expect("tempdir");
        let world = World::open(dir.path()).unwrap();
        world.write("b.txt", b"2", false).unwrap();
        world.write("a.txt", b"1", false).unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        let entries = world.list(".").unwrap();
        let names: Vec<&str> = entries
            .iter()
            .map(|e| e["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "sub"]);
        assert_eq!(entries[2]["type"], json!("directory"));
        std::os::unix::fs::symlink("a.txt", dir.path().join("link")).unwrap();
        assert!(world.list(".").is_err());
    }

    #[test]
    fn snapshot_returns_nested_text_and_detects_changes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let world = World::open(dir.path()).unwrap();
        world.write("top.txt", b"t", true).unwrap();
        world.write("nested/deep.txt", b"d", true).unwrap();
        let snapshot = world.snapshot().unwrap();
        assert_eq!(snapshot["top.txt"], json!("t"));
        assert_eq!(snapshot["nested/deep.txt"], json!("d"));
        world.write("nested/deep.txt", b"x", false).unwrap();
        assert_ne!(world.snapshot().unwrap(), snapshot);
    }
}
