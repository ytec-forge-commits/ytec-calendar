use std::{ptr, slice};
use windows_sys::Win32::{
    Foundation::{GetLastError, ERROR_NOT_FOUND},
    Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    },
};

const TARGET_PREFIX: &str = "Koyomado/GoogleCalendar/";

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn target_name(account_id: &str) -> Vec<u16> {
    wide(&format!("{TARGET_PREFIX}{account_id}"))
}

pub fn store_refresh_token(account_id: &str, refresh_token: &str) -> Result<(), String> {
    if refresh_token.is_empty() {
        return Err("Googleの更新トークンが空です".into());
    }
    let mut target = target_name(account_id);
    let mut username = wide("Koyomado");
    let mut blob = refresh_token.as_bytes().to_vec();
    let credential = CREDENTIALW {
        Flags: 0,
        Type: CRED_TYPE_GENERIC,
        TargetName: target.as_mut_ptr(),
        Comment: ptr::null_mut(),
        LastWritten: unsafe { std::mem::zeroed() },
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        AttributeCount: 0,
        Attributes: ptr::null_mut(),
        TargetAlias: ptr::null_mut(),
        UserName: username.as_mut_ptr(),
    };

    let written = unsafe { CredWriteW(&credential, 0) };
    blob.fill(0);
    if written == 0 {
        return Err(format!(
            "Windows資格情報マネージャーへGoogle認証情報を保存できません（エラー {}）",
            unsafe { GetLastError() }
        ));
    }
    Ok(())
}

pub fn read_refresh_token(account_id: &str) -> Result<Option<String>, String> {
    let target = target_name(account_id);
    let mut raw: *mut CREDENTIALW = ptr::null_mut();
    let read = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut raw) };
    if read == 0 {
        let error = unsafe { GetLastError() };
        if error == ERROR_NOT_FOUND {
            return Ok(None);
        }
        return Err(format!(
            "Windows資格情報マネージャーからGoogle認証情報を読み込めません（エラー {error}）"
        ));
    }
    if raw.is_null() {
        return Ok(None);
    }

    let bytes =
        unsafe { slice::from_raw_parts((*raw).CredentialBlob, (*raw).CredentialBlobSize as usize) };
    let token = String::from_utf8(bytes.to_vec())
        .map_err(|_| "保存済みのGoogle認証情報を読み取れません".to_string());
    unsafe { CredFree(raw.cast()) };
    token.map(Some)
}

pub fn delete_refresh_token(account_id: &str) -> Result<(), String> {
    let target = target_name(account_id);
    let deleted = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
    if deleted == 0 {
        let error = unsafe { GetLastError() };
        if error != ERROR_NOT_FOUND {
            return Err(format!(
                "Windows資格情報マネージャーからGoogle認証情報を削除できません（エラー {error}）"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_target_is_scoped_to_koyomado() {
        let target = String::from_utf16_lossy(&target_name("12345"));
        assert!(target.starts_with("Koyomado/GoogleCalendar/12345"));
    }
}
