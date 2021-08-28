use std::ptr::null_mut;

use winreg::enums::*;
use winreg::RegKey;

use crate::bindings::Windows::Win32::UI::Shell::{
    SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
};

const EXT: &str = ".brs";

const DESCRIPTION: &str = "BRS File";
const CONTENT_TYPE_KEY: &str = "Content Type";
const CONTENT_TYPE_VALUE: &str = "application/brs";
const PERCEIVED_TYPE_KEY: &str = "PerceivedType";
const PERCEIVED_TYPE_VALUE: &str = "image";

const ITHUMBNAILPROVIDER_CLSID: &str = "{e357fccd-a995-4576-b01f-234630154e96}";
const CLSID: &str = "{5f85282f-0cb4-4e7f-a7f1-09fa662b0af0}";

fn shell_change_notify() {
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, null_mut(), null_mut()) };
}

pub fn register_provider() -> Result<(), intercom::raw::HRESULT> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let (key, _) = hkcr.create_subkey(EXT).map_err(|_| intercom::raw::E_FAIL)?;
    key.set_value("", &DESCRIPTION)
        .map_err(|_| intercom::raw::E_FAIL)?;
    key.set_value(CONTENT_TYPE_KEY, &CONTENT_TYPE_VALUE)
        .map_err(|_| intercom::raw::E_FAIL)?;
    key.set_value(PERCEIVED_TYPE_KEY, &PERCEIVED_TYPE_VALUE)
        .map_err(|_| intercom::raw::E_FAIL)?;

    let (shell_ex, _) = key
        .create_subkey("ShellEx")
        .map_err(|_| intercom::raw::E_FAIL)?;

    let (itp_clsid, _) = shell_ex
        .create_subkey(ITHUMBNAILPROVIDER_CLSID)
        .map_err(|_| intercom::raw::E_FAIL)?;

    itp_clsid
        .set_value("", &CLSID)
        .map_err(|_| intercom::raw::E_FAIL)?;

    shell_change_notify();

    Ok(())
}

pub fn unregister_provider() -> Result<(), intercom::raw::HRESULT> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    if let Ok(key) = hkcr.open_subkey(EXT) {
        if let Ok(shell_ex) = key.open_subkey("ShellEx") {
            if let Ok(itp_clsid) =
                shell_ex.open_subkey_with_flags(ITHUMBNAILPROVIDER_CLSID, KEY_READ | KEY_WRITE)
            {
                let rv: Result<String, _> = itp_clsid.get_value("");
                if let Ok(val) = rv {
                    if val == CLSID {
                        itp_clsid
                            .delete_value("")
                            .map_err(|_| intercom::raw::E_FAIL)?;
                    }
                }
            }
        }
    }

    shell_change_notify();

    Ok(())
}
