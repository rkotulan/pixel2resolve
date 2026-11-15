#![windows_subsystem = "windows"]

mod audio;
mod menu;

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::ptr::null_mut;
use windows_sys::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetOpenFileNameW, OFN_DONTADDTORECENT, OFN_EXPLORER, OFN_FILEMUSTEXIST,
    OFN_HIDEREADONLY, OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows_sys::Win32::UI::Controls::{
    TASKDIALOG_BUTTON, TASKDIALOGCONFIG, TDCBF_CLOSE_BUTTON, TDF_USE_COMMAND_LINKS,
    TaskDialogIndirect,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MESSAGEBOX_STYLE, MessageBoxW,
};

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let result = match args.next().as_deref() {
        Some(flag) if flag == "--register-menu" => {
            return exit_status(menu::register(args.next().as_deref()));
        }
        Some(flag) if flag == "--unregister-menu" => return exit_status(menu::unregister()),
        Some(path) => convert_video(Path::new(path)),
        None => show_actions(),
    };
    if let Err(error) = result {
        show_message(&error, MB_ICONERROR);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn exit_status(result: std::io::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn convert_video(input: &Path) -> Result<(), String> {
    let path = audio::convert(input)?;
    show_message(
        &format!(
            "MP4 s převedeným zvukem je připravené.\n\n{}",
            path.display()
        ),
        MB_ICONINFORMATION,
    );
    Ok(())
}

fn show_actions() -> Result<(), String> {
    const OPEN_VIDEO: i32 = 101;
    const REGISTER_MENU: i32 = 102;
    const UNREGISTER_MENU: i32 = 103;

    let registration = menu::registration().ok();
    let (menu_action, menu_button) = match registration {
        Some(menu::Registration::ThisCopy) => (UNREGISTER_MENU, "Odebrat z nabídky Průzkumníku"),
        Some(menu::Registration::OtherCopy) => {
            (REGISTER_MENU, "Používat tuto kopii v nabídce Průzkumníku")
        }
        _ => (REGISTER_MENU, "Přidat do nabídky Průzkumníku"),
    };

    let title = wide("pixel2resolve");
    let instruction = wide("Převod zvuku v MP4");
    let open = wide("Vybrat video MP4");
    let menu_label = wide(menu_button);
    let buttons = [
        TASKDIALOG_BUTTON {
            nButtonID: OPEN_VIDEO,
            pszButtonText: open.as_ptr(),
        },
        TASKDIALOG_BUTTON {
            nButtonID: menu_action,
            pszButtonText: menu_label.as_ptr(),
        },
    ];
    let dialog = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        pszWindowTitle: title.as_ptr(),
        pszMainInstruction: instruction.as_ptr(),
        dwFlags: TDF_USE_COMMAND_LINKS,
        dwCommonButtons: TDCBF_CLOSE_BUTTON,
        cButtons: buttons.len() as u32,
        pButtons: buttons.as_ptr(),
        nDefaultButton: OPEN_VIDEO,
        ..Default::default()
    };
    let mut chosen = 0;
    let status = unsafe { TaskDialogIndirect(&dialog, &mut chosen, null_mut(), null_mut()) };
    if status < 0 {
        return Err(format!(
            "Windows nemohl otevřít okno pixel2resolve ({status:#x})."
        ));
    }

    match chosen {
        OPEN_VIDEO => {
            if let Some(path) = pick_video()? {
                convert_video(&path)?;
            }
        }
        REGISTER_MENU => {
            menu::register(None)
                .map_err(|error| format!("Položku se nepodařilo přidat.\n\n{error}"))?;
            show_message(
                "Položka je přidaná. V Průzkumníku klikni pravým tlačítkem na MP4 a vyber Zobrazit další možnosti.",
                MB_ICONINFORMATION,
            );
        }
        UNREGISTER_MENU => {
            menu::unregister()
                .map_err(|error| format!("Položku se nepodařilo odebrat.\n\n{error}"))?;
            show_message("Položka byla z nabídky odebrána.", MB_ICONINFORMATION);
        }
        _ => {}
    }
    Ok(())
}

fn pick_video() -> Result<Option<PathBuf>, String> {
    let mut filename = vec![0u16; 32768];
    let title = wide("pixel2resolve – vyber MP4");
    let filter = wide("Video MP4\0*.mp4\0");
    let mut dialog = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        lpstrFilter: filter.as_ptr(),
        nFilterIndex: 1,
        lpstrFile: filename.as_mut_ptr(),
        nMaxFile: filename.len() as u32,
        lpstrTitle: title.as_ptr(),
        Flags: OFN_EXPLORER
            | OFN_FILEMUSTEXIST
            | OFN_PATHMUSTEXIST
            | OFN_HIDEREADONLY
            | OFN_NOCHANGEDIR
            | OFN_DONTADDTORECENT,
        ..Default::default()
    };

    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        let error = unsafe { CommDlgExtendedError() };
        return if error == 0 {
            Ok(None)
        } else {
            Err(format!(
                "Windows nemohl otevřít výběr souboru ({error:#x})."
            ))
        };
    }

    let len = filename
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(filename.len());
    Ok(Some(PathBuf::from(OsString::from_wide(&filename[..len]))))
}

fn show_message(message: &str, icon: MESSAGEBOX_STYLE) {
    let message = wide(message);
    let title = wide("pixel2resolve");
    unsafe { MessageBoxW(null_mut(), message.as_ptr(), title.as_ptr(), MB_OK | icon) };
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}
