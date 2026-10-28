//! Windows以外のログイン時の自動起動。macOS=LaunchAgent(`~/Library/LaunchAgents/*.plist`)、Linux=XDG autostart(`~/.config/autostart/*.desktop`)。
//! ファイルの中身の組み立ては純粋関数(どのOSでもテストできる)。実機(macOS/Linux)でのログイン時起動は未確認。

use std::path::{Path, PathBuf};

pub const LABEL: &str = "jp.co.aon.maid-cafe-se";

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// macOSのLaunchAgentのplist。ログイン時に`--minimized`で起動する。
pub fn launch_agent_plist(exe: &Path) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n<dict>\n\
         \x20 <key>Label</key><string>{LABEL}</string>\n\
         \x20 <key>ProgramArguments</key>\n  <array>\n    <string>{}</string>\n    <string>--minimized</string>\n  </array>\n\
         \x20 <key>RunAtLoad</key><true/>\n\
         </dict>\n</plist>\n",
        xml_escape(&exe.display().to_string())
    )
}

/// `.desktop`のExec欄用に、空白などを含むパスを引用する。
fn desktop_quote(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            o.push('\\');
        }
        o.push(c);
    }
    o.push('"');
    o.replace('%', "%%")
}

/// Linuxの自動起動用`.desktop`。
pub fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=maid-cafe-se\nComment=maid-cafe-se (alarm)\nExec={} --minimized\nIcon=maid-cafe-se\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        desktop_quote(&exe.display().to_string())
    )
}

/// 自動起動ファイルの場所。`home`/`xdg_config`は環境から渡す(テストしやすいように)。
pub fn file_path(macos: bool, home: Option<PathBuf>, xdg_config: Option<PathBuf>) -> Option<PathBuf> {
    if macos {
        Some(home?.join("Library/LaunchAgents").join(format!("{LABEL}.plist")))
    } else {
        let base = xdg_config.filter(|p| p.is_absolute()).or_else(|| home.map(|h| h.join(".config")))?;
        Some(base.join("autostart/maid-cafe-se.desktop"))
    }
}

fn current_path() -> Option<PathBuf> {
    file_path(cfg!(target_os = "macos"), std::env::var_os("HOME").map(PathBuf::from), std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
}

pub fn enabled() -> bool {
    current_path().map(|p| p.exists()).unwrap_or(false)
}

pub fn set(on: bool, exe: &Path) -> Result<(), String> {
    let path = current_path().ok_or("ホームフォルダが分かりません")?;
    if on {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("自動起動の設定を変更できませんでした: {e}"))?;
        }
        let body = if cfg!(target_os = "macos") { launch_agent_plist(exe) } else { desktop_entry(exe) };
        std::fs::write(&path, body).map_err(|e| format!("自動起動の設定を変更できませんでした: {e}"))
    } else {
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // 元から無い場合も成功扱い
            Err(e) => Err(format!("自動起動の設定を変更できませんでした: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_has_label_args_and_escapes() {
        let p = launch_agent_plist(Path::new("/Applications/maid & co/maid-cafe-se"));
        assert!(p.contains("<string>jp.co.aon.maid-cafe-se</string>"));
        assert!(p.contains("<string>/Applications/maid &amp; co/maid-cafe-se</string>"));
        assert!(p.contains("<string>--minimized</string>") && p.contains("<key>RunAtLoad</key><true/>"));
    }

    #[test]
    fn desktop_entry_quotes_the_path() {
        let d = desktop_entry(Path::new("/opt/my app/maid-cafe-se"));
        assert!(d.contains("Exec=\"/opt/my app/maid-cafe-se\" --minimized\n"), "{d}");
        assert!(d.starts_with("[Desktop Entry]\n"));
    }

    #[test]
    fn locates_files_per_os() {
        let h = Some(PathBuf::from("/home/u"));
        assert_eq!(Some(PathBuf::from("/home/u/Library/LaunchAgents/jp.co.aon.maid-cafe-se.plist")), file_path(true, h.clone(), None));
        assert_eq!(Some(PathBuf::from("/home/u/.config/autostart/maid-cafe-se.desktop")), file_path(false, h.clone(), None));
        let x = if cfg!(windows) { "C:\\cfg" } else { "/x/cfg" };
        assert_eq!(Some(PathBuf::from(x).join("autostart/maid-cafe-se.desktop")), file_path(false, h, Some(PathBuf::from(x))));
        assert_eq!(None, file_path(false, None, None));
    }
}
