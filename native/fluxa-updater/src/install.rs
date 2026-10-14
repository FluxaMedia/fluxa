use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub enum Applied {
    Restart,
    InstallApk(PathBuf),
}

pub(crate) fn appimage_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    return std::env::var_os("APPIMAGE").map(PathBuf::from);
    #[cfg(not(target_os = "linux"))]
    None
}

#[cfg(target_os = "android")]
pub(crate) fn check_writable() -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub(crate) fn check_writable() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let target = match appimage_path() {
        Some(path) => path,
        None if cfg!(target_os = "macos") => {
            exe.ancestors().nth(3).map(Path::to_path_buf).unwrap_or(exe)
        }
        None => exe,
    };
    let parent = target
        .parent()
        .ok_or("cannot locate the install directory")?;
    let probe = parent.join(".fluxa-update-probe");
    fs::write(&probe, b"").map_err(|_| {
        "Fluxa is installed system-wide. Update it with your package manager or reinstall the new version.".to_owned()
    })?;
    let _ = fs::remove_file(probe);
    Ok(())
}

pub(crate) fn extract(archive: &Path, dest: &Path) -> Result<(), String> {
    let _ = fs::remove_dir_all(dest);
    fs::create_dir_all(dest).map_err(|error| error.to_string())?;
    let name = archive.to_string_lossy();
    if name.ends_with(".apk") {
        fs::copy(archive, dest.join("update.apk"))
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else if name.ends_with(".AppImage") {
        fs::copy(archive, dest.join("Fluxa.AppImage"))
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else if name.ends_with(".tar.gz") {
        let file = fs::File::open(archive).map_err(|error| error.to_string())?;
        tar::Archive::new(flate2::read::GzDecoder::new(file))
            .unpack(dest)
            .map_err(|error| error.to_string())
    } else if name.ends_with(".zip") {
        extract_zip(archive, dest)
    } else {
        Err(format!("unsupported update archive {name}"))
    }
}

#[cfg(target_os = "macos")]
fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    run(Command::new("ditto")
        .arg("-x")
        .arg("-k")
        .arg(archive)
        .arg(dest))
}

#[cfg(not(target_os = "macos"))]
fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(archive).map_err(|error| error.to_string())?;
    zip::ZipArchive::new(file)
        .and_then(|mut zip| zip.extract(dest))
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn run(command: &mut Command) -> Result<(), String> {
    let status = command.status().map_err(|error| error.to_string())?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{:?} failed", command.get_program()))
}

fn single_child(dir: &Path) -> Result<PathBuf, String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|error| error.to_string())?
        .flatten()
        .map(|entry| entry.path());
    match (entries.next(), entries.next()) {
        (Some(only), None) => Ok(only),
        _ => Err("update archive has an unexpected layout".to_owned()),
    }
}

#[cfg(target_os = "android")]
pub(crate) fn apply(staged: &Path) -> Result<Applied, String> {
    Ok(Applied::InstallApk(single_child(staged)?))
}

#[cfg(not(target_os = "android"))]
pub(crate) fn apply(staged: &Path) -> Result<Applied, String> {
    let root = single_child(staged)?;
    if let Some(target) = appimage_path() {
        replace_appimage(&root, &target)?;
        relaunch(&target)?;
        return Ok(Applied::Restart);
    }
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    replace(&root, &exe)?;
    relaunch(&exe)?;
    Ok(Applied::Restart)
}

#[cfg(not(target_os = "android"))]
fn replace_appimage(staged: &Path, target: &Path) -> Result<(), String> {
    let mut name = target.file_name().unwrap_or_default().to_owned();
    name.push(".old");
    let backup = target.with_file_name(name);
    let _ = fs::remove_file(&backup);
    fs::rename(target, &backup)
        .map_err(|error| format!("cannot replace {}: {error}", target.display()))?;
    if let Err(error) = fs::copy(staged, target) {
        let _ = fs::rename(&backup, target);
        return Err(error.to_string());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(target, fs::Permissions::from_mode(0o755));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn replace(staged_app: &Path, exe: &Path) -> Result<(), String> {
    let bundle = exe
        .ancestors()
        .nth(3)
        .filter(|path| path.extension().is_some_and(|ext| ext == "app"))
        .ok_or("Fluxa is not running from an app bundle")?;
    let backup = bundle.with_extension("app.old");
    let _ = fs::remove_dir_all(&backup);
    fs::rename(bundle, &backup)
        .map_err(|error| format!("cannot replace {}: {error}", bundle.display()))?;
    if let Err(error) = run(Command::new("ditto").arg(staged_app).arg(bundle)) {
        let _ = fs::remove_dir_all(bundle);
        let _ = fs::rename(&backup, bundle);
        return Err(error);
    }
    let _ = fs::remove_dir_all(&backup);
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
fn replace(staged_root: &Path, exe: &Path) -> Result<(), String> {
    let install_dir = exe.parent().ok_or("cannot locate the install directory")?;
    copy_tree(staged_root, install_dir)
        .map_err(|error| format!("cannot write to {}: {error}", install_dir.display()))
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&source, &target)?;
        } else if kind.is_symlink() {
            #[cfg(unix)]
            {
                let _ = fs::remove_file(&target);
                std::os::unix::fs::symlink(fs::read_link(&source)?, &target)?;
            }
        } else {
            if target.exists() {
                let mut name = entry.file_name();
                name.push(".old");
                let old = target.with_file_name(name);
                let _ = fs::remove_file(&old);
                fs::rename(&target, &old)?;
            }
            fs::copy(&source, &target)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn relaunch(exe: &Path) -> Result<(), String> {
    let bundle = exe
        .ancestors()
        .nth(3)
        .ok_or("cannot locate the app bundle")?;
    Command::new("open")
        .arg("-n")
        .arg(bundle)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
fn relaunch(exe: &Path) -> Result<(), String> {
    Command::new(exe)
        .args(std::env::args_os().skip(1))
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "android")]
pub(crate) fn remove_leftovers() {}

#[cfg(not(target_os = "android"))]
pub(crate) fn remove_leftovers() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let Some(bundle) = exe.ancestors().nth(3) {
        let _ = fs::remove_dir_all(bundle.with_extension("app.old"));
    }
    #[cfg(not(target_os = "macos"))]
    if let Some(dir) = exe.parent() {
        remove_old_files(dir, 3);
    }
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
fn remove_old_files(dir: &Path, depth: u8) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() && depth > 0 {
            remove_old_files(&path, depth - 1);
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "old") {
            let _ = fs::remove_file(&path);
        }
    }
}

#[cfg(all(test, not(any(target_os = "macos", target_os = "android"))))]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fluxa-updater-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn replaced_binary_keeps_a_removable_backup() {
        let dir = scratch("replace");
        let staged = dir.join("staged/fluxa");
        let installed = dir.join("installed");
        fs::create_dir_all(staged.join("lib")).unwrap();
        fs::create_dir_all(&installed).unwrap();
        fs::write(installed.join("fluxa-desktop"), "old").unwrap();
        fs::write(staged.join("fluxa-desktop"), "new").unwrap();
        fs::write(staged.join("lib/libmpv.so"), "mpv").unwrap();

        copy_tree(&staged, &installed).unwrap();

        assert_eq!(
            fs::read_to_string(installed.join("fluxa-desktop")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(installed.join("fluxa-desktop.old")).unwrap(),
            "old"
        );
        assert!(installed.join("lib/libmpv.so").exists());
        remove_old_files(&installed, 3);
        assert!(!installed.join("fluxa-desktop.old").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn archive_with_two_top_level_entries_is_rejected() {
        let dir = scratch("layout");
        fs::create_dir_all(dir.join("a")).unwrap();
        fs::create_dir_all(dir.join("b")).unwrap();
        assert!(single_child(&dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}
