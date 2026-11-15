use std::env;
use std::io::ErrorKind;
use std::os::windows::process::CommandExt;
use std::path::{self, Path, PathBuf};
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn convert(input: &Path) -> Result<PathBuf, String> {
    let input =
        path::absolute(input).map_err(|error| format!("Neplatná cesta k videu.\n\n{error}"))?;
    if !input.is_file() {
        return Err(format!("Video nebylo nalezeno.\n\n{}", input.display()));
    }
    if !input
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("mp4"))
    {
        return Err("Vyber video ve formátu MP4.".into());
    }

    let folder = input
        .parent()
        .ok_or("Nepodařilo se zjistit složku videa.")?;
    let stem = input
        .file_stem()
        .ok_or("Nepodařilo se zjistit název videa.")?;
    let ffmpeg =
        find_ffmpeg().ok_or("Soubor ffmpeg.exe musí být vedle pixel2resolve.exe nebo v PATH.")?;
    let temporary = tempfile::Builder::new()
        .prefix(".pixel2resolve-")
        .suffix(".tmp")
        .tempfile_in(folder)
        .map_err(|error| format!("Ve složce videa nelze vytvořit nové MP4.\n\n{error}"))?
        .into_temp_path();

    let result = Command::new(ffmpeg)
        .creation_flags(CREATE_NO_WINDOW)
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(&input)
        .args([
            "-map",
            "0:v",
            "-map",
            "0:a",
            "-map",
            "0:s?",
            "-map",
            "0:d?",
            "-map",
            "0:t?",
            "-c",
            "copy",
            "-c:a",
            "aac",
            "-profile:a",
            "aac_low",
            "-ar:a",
            "48000",
            "-b:a",
            "192k",
            "-abort_on",
            "empty_output_stream",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ])
        .arg(temporary.as_os_str())
        .output()
        .map_err(|error| format!("FFmpeg se nepodařilo spustit.\n\n{error}"))?;

    if !result.status.success() {
        let details = String::from_utf8_lossy(&result.stderr);
        if details.contains("matches no streams") {
            if details.contains("'0:a'") {
                return Err("Video neobsahuje zvukovou stopu.".into());
            }
            if details.contains("'0:v'") {
                return Err("Soubor neobsahuje obrazovou stopu.".into());
            }
        }
        return Err(format!("Převod zvuku se nezdařil.\n\n{}", details.trim()));
    }

    let mut temporary = temporary;
    let mut sequence = 1;
    loop {
        let mut name = stem.to_os_string();
        if sequence > 1 {
            name.push(format!(" ({sequence})"));
        }
        name.push(".p2r.mp4");
        let output = folder.join(name);
        match temporary.persist_noclobber(&output) {
            Ok(()) => return Ok(output),
            Err(error) if error.error.kind() == ErrorKind::AlreadyExists => {
                temporary = error.path;
                sequence += 1;
            }
            Err(error) => return Err(format!("MP4 se nepodařilo uložit.\n\n{}", error.error)),
        }
    }
}

fn find_ffmpeg() -> Option<PathBuf> {
    let executable = env::current_exe().ok()?;
    let bundled = executable.with_file_name("ffmpeg.exe");
    if bundled.is_file() {
        return Some(bundled);
    }

    let paths = env::var_os("PATH")?;
    env::split_paths(&paths)
        .filter(|path| path.is_absolute())
        .map(|path| path.join("ffmpeg.exe"))
        .find(|path| path.is_file())
}
