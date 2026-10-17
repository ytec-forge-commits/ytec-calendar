use std::path::{Path, PathBuf};

fn resolve_manual_path(executable: &Path, packaged: bool) -> Result<PathBuf, String> {
    let directory = executable
        .parent()
        .ok_or("アプリの配置を確認できません。")?;
    let directory = if packaged {
        directory
            .parent()
            .ok_or("アプリの配置を確認できません。")?
            .join("docs")
    } else {
        directory.to_path_buf()
    };
    let manual = directory.join("Koyomado操作説明書.pdf");
    if !manual.is_file() {
        return Err("同梱の操作説明書が見つかりません。アプリをフォルダー全体で展開し直してください。Store版は再インストール前に予定データをバックアップしてください。".into());
    }
    Ok(manual)
}

#[tauri::command]
pub(crate) fn open_user_manual() -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|_| "アプリの配置を確認できません。")?;
    let manual = resolve_manual_path(&executable, super::current_process_has_package_identity()?)?;
    open_pdf(&manual)
}

#[cfg(windows)]
fn open_pdf(path: &Path) -> Result<(), String> {
    use std::{os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};
    // A fixed bundled filename only: no caller-supplied URL, path, or shell command.
    let target: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            ptr::null(),
            target.as_ptr(),
            ptr::null(),
            ptr::null(),
            SW_SHOWNORMAL,
        )
    } as isize;
    if result <= 32 {
        return Err(format!("操作説明書を開けませんでした。PDFを開く既定のアプリをWindowsで確認してください（エラー {result}）。"));
    }
    Ok(())
}

#[cfg(not(windows))]
fn open_pdf(_path: &Path) -> Result<(), String> {
    Err("操作説明書はWindowsアプリから開いてください。".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn fixture() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("koyomado-manual-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn portable_manual_is_read_from_the_executable_folder_with_unicode_and_spaces() {
        let root = fixture().join("Portable & 日本語");
        fs::create_dir_all(&root).unwrap();
        let pdf = root.join("Koyomado操作説明書.pdf");
        fs::write(&pdf, b"%PDF-1.7\nsynthetic").unwrap();
        assert_eq!(
            resolve_manual_path(&root.join("koyomado.exe"), false).unwrap(),
            pdf
        );
    }

    #[test]
    fn store_manual_is_read_from_package_docs_instead_of_a_portable_decoy() {
        let root = fixture();
        fs::create_dir_all(root.join("app")).unwrap();
        fs::create_dir_all(root.join("docs")).unwrap();
        let pdf = root.join("docs/Koyomado操作説明書.pdf");
        fs::write(&pdf, b"%PDF-1.7\nsynthetic").unwrap();
        fs::write(root.join("app/Koyomado操作説明書.pdf"), b"decoy").unwrap();
        assert_eq!(
            resolve_manual_path(&root.join("app/koyomado.exe"), true).unwrap(),
            pdf
        );
    }

    #[test]
    fn missing_or_directory_manual_fails_without_creating_data() {
        let root = fixture();
        fs::create_dir(root.join("Koyomado操作説明書.pdf")).unwrap();
        assert!(resolve_manual_path(&root.join("koyomado.exe"), false).is_err());
        assert!(!root.join("data").exists());
        assert!(resolve_manual_path(&root.join("app/koyomado.exe"), true).is_err());
        assert!(!root.join("docs").exists());
    }
}
