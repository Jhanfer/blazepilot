use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;
use crate::core::system::trash_manager::{
    error::{TrashError, TrashResult},
    manager::{TrashBackend, TrashDestination},
};
use chrono::{DateTime, Utc};
use std::{
    fs, io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
};
use users::get_current_uid;

#[derive(Debug, Clone)]
pub struct LinuxTrashBackend {
    home_dir: Arc<Path>,
    uid: u32,
}

impl LinuxTrashBackend {
    pub fn new() -> TrashResult<Self> {
        let home = KnownDirsManager::get().home.clone();
        let uid = get_current_uid();
        Ok(Self {
            home_dir: home,
            uid,
        })
    }

    fn home_trash_root(&self) -> PathBuf {
        self.home_dir.join(".local").join("share").join("Trash")
    }

    fn home_trash_files(&self) -> PathBuf {
        self.home_dir
            .join(".local")
            .join("share")
            .join("Trash")
            .join("files")
    }

    fn external_trash_root(&self, mount_point: Arc<Path>) -> TrashResult<Arc<Path>> {
        let dot_trash = mount_point.join(".Trash");
        let dot_trash_uid = mount_point.join(format!(".Trash-{}", self.uid));

        if dot_trash.is_dir() {
            match fs::metadata(&dot_trash) {
                Ok(meta) => {
                    let perms = meta.permissions().mode();

                    if (perms & 0o1000) != 0 && (perms & 0o002) == 0 {
                        let user_trash = dot_trash.join(self.uid.to_string());

                        if !user_trash.exists() {
                            fs::create_dir_all(&user_trash).map_err(TrashError::Io)?;

                            fs::set_permissions(&user_trash, fs::Permissions::from_mode(0o700))
                                .map_err(TrashError::Io)?;
                        }

                        return Ok(user_trash.as_path().into());
                    } else {
                        return Err(TrashError::TrashDirInvalidPermissions {
                            path: dot_trash.clone(),
                        });
                    }
                }

                Err(e) => {
                    return Err(TrashError::Io(e));
                }
            }
        }

        if !dot_trash_uid.exists() {
            fs::create_dir_all(&dot_trash_uid).map_err(TrashError::Io)?;

            fs::set_permissions(&dot_trash_uid, fs::Permissions::from_mode(0o700))
                .map_err(TrashError::Io)?;
        }

        Ok(dot_trash_uid.as_path().into())
    }

    fn files_dir(&self, root: &Path) -> PathBuf {
        root.join("files")
    }

    fn info_dir(&self, root: &Path) -> PathBuf {
        root.join("info")
    }

    fn generate_trashinfo(&self, original_path: &Path, deletion_date: DateTime<Utc>) -> String {
        //Genera el contenido de trash info
        let original_uri = Self::path_to_uri(original_path);
        format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            original_uri,
            deletion_date.format("%Y-%m-%dT%H:%M:%S")
        )
    }

    fn detect_mount_point(path: &Path) -> TrashResult<Arc<Path>> {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        //Detección de puntos de montaje
        if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
            let mut best_match: Option<PathBuf> = None;

            for line in mounts.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();

                if parts.len() < 2 {
                    continue;
                }

                let mount = PathBuf::from(parts[1]);

                if canonical.starts_with(&mount) {
                    match &best_match {
                        Some(current_best) => {
                            if mount.components().count() > current_best.components().count() {
                                best_match = Some(mount);
                            }
                        }
                        None => {
                            best_match = Some(mount);
                        }
                    }
                }
            }

            if let Some(best) = best_match {
                return Ok(best.into());
            }
        }

        // fallback basado en device ID
        let mut current = canonical;

        let target_dev = fs::metadata(&current).map_err(TrashError::Io)?.dev();

        while let Some(parent) = current.parent() {
            let parent_dev = fs::metadata(parent).map_err(TrashError::Io)?.dev();
            if parent_dev != target_dev {
                return Ok(current.into());
            }
            current = parent.to_path_buf();
        }

        Ok(current.into())
    }

    fn ensure_writable(path: &Path) -> TrashResult<()> {
        //crear un archivo para verificar si es escribible
        let test_file = path.join(".blazepilot_write_probe");
        fs::write(&test_file, b"").map_err(|_| TrashError::DirNotWritable {
            path: path.to_path_buf(),
        })?;
        let _ = fs::remove_file(test_file);
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------

    fn path_to_uri(path: &Path) -> String {
        let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let path_str = abs.to_string_lossy();
        let encoded = path_str
            .split("/")
            .map(|sgm| urlencoding::encode(sgm))
            .collect::<Vec<_>>()
            .join("/");

        format!("file://{}", encoded)
    }

    fn find_trash_root_for_file(file: &Path, uid: u32) -> TrashResult<PathBuf> {
        let uid_str = uid.to_string();
        let mut current = file;

        while let Some(parent) = current.parent() {
            if parent.file_name() == Some(std::ffi::OsStr::new("files")) {
                if let Some(root) = parent.parent() {
                    let root_name = root.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let is_home_trash = root.ends_with(".local/share/Trash");
                    let is_uid_trash =
                        root_name == format!(".Trash-{}", uid_str) || root_name == uid_str;

                    if is_home_trash || is_uid_trash {
                        return Ok(root.to_path_buf());
                    }
                }
            }
            current = parent;
        }

        Err(TrashError::TrashEntryNotFound {
            path: file.to_path_buf(),
        })
    }

    fn parse_trash_info_path(content: &str) -> Option<PathBuf> {
        for line in content.lines() {
            if line.starts_with("Path=") {
                let uri = line.strip_prefix("Path=")?;
                if uri.starts_with("file://") {
                    let path_part = uri.strip_prefix("file://")?;
                    let decoded = urlencoding::decode(path_part).ok()?;
                    return Some(PathBuf::from(decoded.into_owned()));
                }
            }
        }
        None
    }

    fn resolve_name_collision(path: &Path) -> PathBuf {
        if !path.exists() {
            return path.to_owned();
        }

        let stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let timestamp = Utc::now().timestamp();
        let mut candidate = path.with_file_name(format!("{}_{}.{}", stem, timestamp, ext));

        let mut counter = 0;

        while candidate.exists() {
            candidate = path.with_file_name(format!("{}_{}_{}.{}", stem, timestamp, counter, ext));
            counter += 1;
        }

        candidate
    }

    fn safe_remove_dir_contents(dir: &Path) -> TrashResult<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(TrashError::Io)? {
            let entry = entry.map_err(TrashError::Io)?;
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path).map_err(TrashError::Io)?;
            } else {
                fs::remove_file(&path).map_err(TrashError::Io)?;
            }
        }

        Ok(())
    }

    fn copy_dir_recursive(src: &Path, dst: &Path) -> TrashResult<()> {
        fs::create_dir_all(dst).map_err(TrashError::Io)?;

        let root_perms = fs::metadata(src).map_err(TrashError::Io)?.permissions();

        fs::set_permissions(dst, root_perms).map_err(TrashError::Io)?;

        for entry in fs::read_dir(src).map_err(TrashError::Io)? {
            let entry = entry.map_err(TrashError::Io)?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            let meta = fs::symlink_metadata(&src_path).map_err(TrashError::Io)?;

            if meta.is_symlink() {
                let resolved = fs::canonicalize(&src_path).map_err(TrashError::Io)?;
                let resolved_meta = fs::metadata(&resolved).map_err(TrashError::Io)?;

                if resolved.is_dir() {
                    Self::copy_dir_recursive(&resolved, &dst_path)?;
                } else {
                    fs::copy(&resolved, &dst_path).map_err(TrashError::Io)?;
                    fs::set_permissions(&dst_path, resolved_meta.permissions())
                        .map_err(TrashError::Io)?;
                }
            } else if meta.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path).map_err(TrashError::Io)?;

                fs::set_permissions(&dst_path, meta.permissions()).map_err(TrashError::Io)?;
            }
        }

        Ok(())
    }
}

impl TrashBackend for LinuxTrashBackend {
    fn etched_in_trash_path(&self, path: &Path) -> bool {
        if path == self.home_trash_files() {
            return true;
        }

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == format!(".Trash-{}", self.uid) {
                return true;
            }
        }

        if let (Some(name), Some(parent_name)) = (
            path.file_name().and_then(|n| n.to_str()),
            path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str()),
        ) {
            if name == self.uid.to_string() && parent_name == ".Trash" {
                return true;
            }
        }

        false
    }

    fn is_in_trash(&self, path: &Path) -> bool {
        Self::find_trash_root_for_file(path, self.uid).is_ok()
    }

    fn get_trash_files(&self, destination: &TrashDestination) -> TrashResult<Arc<Path>> {
        match destination {
            TrashDestination::Home => {
                let root = self.home_trash_files();
                fs::create_dir_all(&root).map_err(TrashError::Io)?;
                Ok(root.into())
            }
            TrashDestination::External { mount_point } => {
                self.external_trash_root(mount_point.to_owned())
            }
        }
    }

    fn get_trash_root(&self, destination: &TrashDestination) -> TrashResult<Arc<Path>> {
        match destination {
            TrashDestination::Home => {
                let root = self.home_trash_root();
                fs::create_dir_all(self.files_dir(&root)).map_err(TrashError::Io)?;
                fs::create_dir_all(self.info_dir(&root)).map_err(TrashError::Io)?;
                Ok(root.into())
            }
            TrashDestination::External { mount_point } => {
                self.external_trash_root(mount_point.to_owned())
            }
        }
    }

    fn resolve_destination(&self, file_path: &Path) -> TrashResult<TrashDestination> {
        //busca de maanera dinámica la papelera que le toca a cada file
        let resolved = if file_path.exists() {
            file_path
                .canonicalize()
                .unwrap_or_else(|_| file_path.to_path_buf())
        } else {
            if let Some(parent) = file_path.parent() {
                parent
                    .canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf())
                    .join(file_path.file_name().unwrap_or_default())
            } else {
                file_path.to_path_buf()
            }
        };

        let home_dev = fs::metadata(&self.home_dir).map_err(TrashError::Io)?.dev();
        let file_dev = fs::metadata(&resolved).map_err(TrashError::Io)?.dev();

        if home_dev == file_dev {
            Ok(TrashDestination::Home)
        } else {
            let mount_point = Self::detect_mount_point(&resolved)?;
            Ok(TrashDestination::External { mount_point })
        }
    }

    fn move_to_trash(&self, source: &Path) -> TrashResult<PathBuf> {
        if !source.exists() {
            return Err(TrashError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "El archivo no existe",
            )));
        }

        let destination = self.resolve_destination(source)?;
        let trash_root = self.get_trash_root(&destination)?;
        let files_dir = self.files_dir(&trash_root);
        let info_dir = self.info_dir(&trash_root);

        fs::create_dir_all(&files_dir).map_err(TrashError::Io)?;
        fs::create_dir_all(&info_dir).map_err(TrashError::Io)?;
        Self::ensure_writable(&files_dir)?;
        Self::ensure_writable(&info_dir)?;

        let original_name = source
            .file_name()
            .ok_or_else(|| TrashError::InvalidPath(source.to_path_buf()))?;
        let mut trash_name = original_name.to_os_string();
        let mut counter = 0;

        loop {
            let candidate = files_dir.join(&trash_name);
            if !candidate.exists() {
                break;
            }

            let name_str = original_name.to_string_lossy();
            if let Some((stem, ext)) = name_str.rsplit_once(".") {
                trash_name = format!("{}_{}.{}", stem, counter, ext).into();
            } else {
                trash_name = format!("{}_{}", name_str, counter).into();
            }
            counter += 1;
        }

        let trash_path = files_dir.join(&trash_name);
        let info_name = format!("{}.trashinfo", trash_name.to_string_lossy());
        let info_path = info_dir.join(info_name);

        let deletion_date = Utc::now();
        let trashinfo_content = self.generate_trashinfo(source, deletion_date);

        fs::write(&info_path, trashinfo_content).map_err(TrashError::Io)?;
        fs::set_permissions(&info_path, fs::Permissions::from_mode(0o600))
            .map_err(TrashError::Io)?;

        if let Err(e) = fs::rename(source, &trash_path) {
            if e.kind() == io::ErrorKind::CrossesDevices {
                if source.is_dir() {
                    Self::copy_dir_recursive(source, &trash_path)?;
                    fs::remove_dir_all(source).map_err(TrashError::Io)?;
                } else {
                    fs::copy(source, &trash_path).map_err(TrashError::Io)?;
                    fs::set_permissions(
                        &trash_path,
                        fs::metadata(source).map_err(TrashError::Io)?.permissions(),
                    )
                    .map_err(TrashError::Io)?;
                    fs::remove_file(source).map_err(TrashError::Io)?;
                }
            }
        }

        Ok(trash_path)
    }

    fn restore_from_trash(&self, trash_path: &Path) -> TrashResult<PathBuf> {
        let trash_root = Self::find_trash_root_for_file(trash_path, self.uid)?;
        let info_dir = self.info_dir(&trash_root);
        let trash_name = trash_path
            .file_name()
            .ok_or_else(|| TrashError::InvalidPath(trash_path.to_path_buf()))?;

        let info_name = format!("{}.trashinfo", trash_name.to_string_lossy());
        let info_path = info_dir.join(&info_name);

        if !info_path.exists() {
            return Err(TrashError::TrashEntryNotFound {
                path: trash_path.to_path_buf(),
            });
        }

        let content = fs::read_to_string(&info_path).map_err(TrashError::Io)?;
        let original_path =
            Self::parse_trash_info_path(&content).ok_or(TrashError::RestoreInvalidName)?;

        if let Some(parent) = original_path.parent() {
            fs::create_dir_all(parent).map_err(TrashError::Io)?;
        }

        let final_path = Self::resolve_name_collision(&original_path);
        fs::rename(trash_path, &final_path).map_err(TrashError::Io)?;

        if info_path.exists() {
            fs::remove_file(&info_path).map_err(TrashError::Io)?;
        }

        Ok(final_path)
    }

    fn permanently_delete(&self, trash_path: &Path) -> TrashResult<()> {
        let trash_root = Self::find_trash_root_for_file(trash_path, self.uid)?;
        let info_dir = self.info_dir(&trash_root);
        let files_dir = self.files_dir(&trash_root);

        if !trash_path.starts_with(&files_dir) {
            return Err(TrashError::TrashEntryNotFound {
                path: trash_path.to_owned(),
            });
        }

        let trash_name = trash_path
            .file_name()
            .ok_or_else(|| TrashError::InvalidPath(trash_path.to_path_buf()))?;
        let info_name = format!("{}.trashinfo", trash_name.to_string_lossy());
        let info_path = info_dir.join(info_name);

        if trash_path.is_dir() {
            fs::remove_dir_all(trash_path).map_err(TrashError::Io)?;
        } else {
            fs::remove_file(trash_path).map_err(TrashError::Io)?;
        }

        if info_path.exists() {
            fs::remove_file(&info_path).map_err(TrashError::Io)?;
        }

        Ok(())
    }

    fn empty_trash(&self) -> TrashResult<()> {
        let home_root = self.home_trash_root();
        if home_root.exists() {
            Self::safe_remove_dir_contents(&self.files_dir(&home_root))?;
            Self::safe_remove_dir_contents(&self.info_dir(&home_root))?;
        }
        Ok(())
    }
}
