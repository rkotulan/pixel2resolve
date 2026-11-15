use std::env;
use std::ffi::{OsStr, OsString};
use std::io::{self, ErrorKind};
use std::ptr::null;
use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use winreg::HKCU;

const VERB: &str = r"Software\Classes\SystemFileAssociations\.mp4\shell\pixel2resolve.ConvertAudio";
const DEFAULT_LABEL: &str = "pixel2resolve – převést zvuk v MP4";

pub enum Registration {
    Missing,
    ThisCopy,
    OtherCopy,
}

pub fn registration() -> io::Result<Registration> {
    let current = HKCU
        .open_subkey(format!(r"{VERB}\command"))
        .and_then(|key| key.get_value::<OsString, _>(""));
    match current {
        Ok(command) if command == shell_command()? => Ok(Registration::ThisCopy),
        Ok(_) => Ok(Registration::OtherCopy),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Registration::Missing),
        Err(error) => Err(error),
    }
}

pub fn register(label: Option<&OsStr>) -> io::Result<()> {
    let command = shell_command()?;
    let (key, _) = HKCU.create_subkey(VERB)?;
    let label = label
        .filter(|label| !label.is_empty())
        .map(OsStr::to_os_string)
        .or_else(|| key.get_value::<OsString, _>("MUIVerb").ok())
        .unwrap_or_else(|| DEFAULT_LABEL.into());
    key.set_value("MUIVerb", &label)?;
    key.set_value("MultiSelectModel", &"Single")?;
    let (command_key, _) = key.create_subkey("command")?;
    command_key.set_value("", &command)?;
    refresh_explorer();
    Ok(())
}

pub fn unregister() -> io::Result<()> {
    match registration()? {
        Registration::Missing => Ok(()),
        Registration::ThisCopy => {
            HKCU.delete_subkey_all(VERB)?;
            refresh_explorer();
            Ok(())
        }
        Registration::OtherCopy => Err(io::Error::other(
            "Položka v nabídce patří jiné kopii pixel2resolve.",
        )),
    }
}

fn shell_command() -> io::Result<OsString> {
    let mut command = OsString::from("\"");
    command.push(env::current_exe()?.as_os_str());
    command.push("\" \"%1\"");
    Ok(command)
}

fn refresh_explorer() {
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED as i32, SHCNF_IDLIST, null(), null()) };
}
