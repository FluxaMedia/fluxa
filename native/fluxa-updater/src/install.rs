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

pub(crate) fn system_package() -> Option<&'static str> {
    #[cfg(target_os = "linux")]
    {
        if appimage_path().is_some() {
            return None;
        }
        let exe = std::env::current_exe().ok()?;
        let owns = |tool: &str, flag: &str| {
            Command::new(tool)
                .arg(flag)
                .arg(&exe)
                .output()
                .is_ok_and(|output| output.status.success())
        };
        if owns("dpkg", "-S") {
            Some("deb")
        } else if owns("rpm", "-qf") {
            Some("rpm")
        } else {
            None
        }
    }
    #[cfg(target_os = "windows")]
    {
        let exe = std::env::current_exe().ok()?;
        exe.with_file_name("unins000.exe")
            .is_file()
            .then_some("setup")
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    None
}

#[cfg(target_os = "android")]
pub(crate) fn check_writable() -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "android"))]
pub(crate) fn check_writable() -> Result<(), String> {
    if system_package().is_some() {
        return Ok(());
    }
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
    } else if name.ends_with(".exe") {
        fs::copy(archive, dest.join("update.exe"))
            .map(|_| ())
            .map_err(|error| error.to_string())
    } else if name.ends_with(".deb") || name.ends_with(".rpm") {
        let ext = if name.ends_with(".deb") { "deb" } else { "rpm" };
        fs::copy(archive, dest.join(format!("update.{ext}")))
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
    match system_package() {
        Some("setup") => {
            run_setup(&root)?;
            return Ok(Applied::Restart);
        }
        Some(kind) => install_package(&root, kind)?,
        None => replace(&root, &exe)?,
    }
    relaunch(&exe)?;
    Ok(Applied::Restart)
}

#[cfg(windows)]
fn run_setup(setup: &Path) -> Result<(), String> {
    let runner = std::env::temp_dir().join("fluxa-update-setup.exe");
    fs::copy(setup, &runner).map_err(|error| error.to_string())?;
    Command::new(&runner)
        .args([
            "/VERYSILENT",
            "/SUPPRESSMSGBOXES",
            "/NORESTART",
            "/CLOSEAPPLICATIONS",
        ])
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(all(not(windows), not(target_os = "android")))]
fn run_setup(_: &Path) -> Result<(), String> {
    Err("the setup installer only applies on Windows".to_owned())
}

#[cfg(not(target_os = "android"))]
fn install_package(file: &Path, kind: &str) -> Result<(), String> {
    let (tool, flag) = if kind == "deb" {
        ("dpkg", "-i")
    } else {
        ("rpm", "-U")
    };
    let status = Command::new("pkexec")
        .arg(tool)
        .arg(flag)
        .arg(file)
        .status()
        .map_err(|_| {
            "pkexec is not available, install the downloaded package manually".to_owned()
        })?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "the package install was cancelled or failed".to_owned())
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
    if let Some(target) = appimage_path() {
        let _ = fs::remove_file(sibling(&target, "old"));
        let _ = fs::remove_file(sibling(&target, "bad"));
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let Some(bundle) = exe.ancestors().nth(3) {
        let _ = fs::remove_dir_all(bundle.with_extension("app.old"));
        let _ = fs::remove_dir_all(bundle.with_extension("app.bad"));
    }
    #[cfg(not(target_os = "macos"))]
    if let Some(dir) = exe.parent() {
        remove_old_files(dir, 3);
    }
}

#[cfg(not(target_os = "android"))]
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_owned();
    name.push(".");
    name.push(suffix);
    path.with_file_name(name)
}

#[cfg(target_os = "android")]
pub(crate) fn keeps_backup() -> bool {
    false
}

#[cfg(not(target_os = "android"))]
pub(crate) fn keeps_backup() -> bool {
    appimage_path().is_some() || system_package().is_none()
}

#[cfg(target_os = "android")]
pub(crate) fn restore() -> Result<(), String> {
    Err("rollback is not available on Android".to_owned())
}

#[cfg(not(target_os = "android"))]
pub(crate) fn restore() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    if let Some(target) = appimage_path() {
        swap_back(&target)?;
        return relaunch(&target);
    }
    #[cfg(target_os = "macos")]
    {
        let bundle = exe
            .ancestors()
            .nth(3)
            .ok_or("cannot locate the app bundle")?;
        let backup = bundle.with_extension("app.old");
        if !backup.is_dir() {
            return Err("no previous version to restore".to_owned());
        }
        let bad = bundle.with_extension("app.bad");
        let _ = fs::remove_dir_all(&bad);
        fs::rename(bundle, &bad).map_err(|error| error.to_string())?;
        fs::rename(&backup, bundle).map_err(|error| error.to_string())?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let dir = exe.parent().ok_or("cannot locate the install directory")?;
        if restore_old_files(dir, 3)? == 0 {
            return Err("no previous version to restore".to_owned());
        }
    }
    relaunch(&exe)
}

#[cfg(not(target_os = "android"))]
fn swap_back(target: &Path) -> Result<(), String> {
    let backup = sibling(target, "old");
    if !backup.is_file() {
        return Err("no previous version to restore".to_owned());
    }
    let bad = sibling(target, "bad");
    let _ = fs::remove_file(&bad);
    fs::rename(target, &bad).map_err(|error| error.to_string())?;
    fs::rename(&backup, target).map_err(|error| error.to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "android")))]
fn restore_old_files(dir: &Path, depth: u8) -> Result<usize, String> {
    let mut restored = 0;
    for entry in fs::read_dir(dir)
        .map_err(|error| error.to_string())?
        .flatten()
    {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() && depth > 0 {
            restored += restore_old_files(&path, depth - 1)?;
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "old") {
            let target = path.with_extension("");
            if target.exists() {
                let bad = sibling(&target, "bad");
                let _ = fs::remove_file(&bad);
                fs::rename(&target, &bad).map_err(|error| error.to_string())?;
            }
            fs::rename(&path, &target).map_err(|error| error.to_string())?;
            restored += 1;
        }
    }
    Ok(restored)
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
        } else if kind.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "old" || ext == "bad")
        {
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
    fn rollback_puts_the_old_files_back() {
        let dir = scratch("restore");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::write(dir.join("fluxa-desktop"), "new").unwrap();
        fs::write(dir.join("fluxa-desktop.old"), "old").unwrap();
        fs::write(dir.join("lib/libmpv.so"), "new mpv").unwrap();
        fs::write(dir.join("lib/libmpv.so.old"), "old mpv").unwrap();

        assert_eq!(restore_old_files(&dir, 3).unwrap(), 2);

        assert_eq!(
            fs::read_to_string(dir.join("fluxa-desktop")).unwrap(),
            "old"
        );
        assert_eq!(
            fs::read_to_string(dir.join("lib/libmpv.so")).unwrap(),
            "old mpv"
        );
        assert_eq!(
            fs::read_to_string(dir.join("fluxa-desktop.bad")).unwrap(),
            "new"
        );
        remove_old_files(&dir, 3);
        assert!(!dir.join("fluxa-desktop.bad").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn deb_and_rpm_are_staged_under_a_fixed_name() {
        let dir = scratch("packages");
        for ext in ["deb", "rpm"] {
            let archive = dir.join(format!("fluxa-0.2.1.{ext}"));
            fs::write(&archive, "pkg").unwrap();
            let staged = dir.join(format!("staged-{ext}"));
            extract(&archive, &staged).unwrap();
            assert_eq!(
                single_child(&staged)
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy(),
                format!("update.{ext}")
            );
        }
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
