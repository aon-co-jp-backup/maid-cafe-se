//! Windows以外の再生。WAVを一時ファイルに書き、OSのプレイヤーに鳴らさせる(macOS=`afplay`、Linux=`paplay`/`pw-play`/`aplay`/`ffplay`の最初に使えたもの)。
//! 同時に鳴らすのは1つだけ(`Player`が直列化する)ので、再生中のプロセスは1つの置き場に持つ。
//!
//! 実機の確認状況: macOS/Linuxの実機では未確認(開発機がWindowsのため)。

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

struct Current {
    child: Child,
    file: PathBuf,
}

static CURRENT: Mutex<Option<Current>> = Mutex::new(None);

/// このOSで試すプレイヤー(コマンド名と引数)。ファイル名は最後に付ける。
pub fn players() -> Vec<(&'static str, Vec<&'static str>)> {
    if cfg!(target_os = "macos") {
        vec![("afplay", vec![])]
    } else {
        vec![
            ("paplay", vec![]),
            ("pw-play", vec![]),
            ("aplay", vec!["-q"]),
            ("ffplay", vec!["-nodisp", "-autoexit", "-loglevel", "quiet"]),
        ]
    }
}

fn temp_wav() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    std::env::temp_dir().join(format!("maid-cafe-se-play-{}-{}.wav", std::process::id(), N.fetch_add(1, Ordering::Relaxed)))
}

fn start(wav: &[u8]) -> bool {
    let file = temp_wav();
    if std::fs::write(&file, wav).is_err() {
        return false;
    }
    for (prog, args) in players() {
        if let Ok(child) = Command::new(prog).args(args).arg(&file).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn() {
            *CURRENT.lock().unwrap_or_else(|e| e.into_inner()) = Some(Current { child, file });
            return true;
        }
    }
    let _ = std::fs::remove_file(&file);
    false
}

/// WAVを最後まで鳴らす。他スレッドから[`stop_all`]されたら途中で返る。プレイヤーが無い等で鳴らせなければ`false`。
pub fn play_sync(wav: &[u8]) -> bool {
    if !start(wav) {
        return false;
    }
    loop {
        {
            let mut g = CURRENT.lock().unwrap_or_else(|e| e.into_inner());
            match g.as_mut() {
                None => return true, // stop_allされた
                Some(c) => match c.child.try_wait() {
                    Ok(None) => {}
                    Ok(Some(s)) => {
                        let _ = std::fs::remove_file(&c.file);
                        *g = None;
                        return s.success();
                    }
                    Err(_) => {
                        let _ = std::fs::remove_file(&c.file);
                        *g = None;
                        return false;
                    }
                },
            }
        }
        std::thread::sleep(Duration::from_millis(30));
    }
}

/// WAVを鳴らし始める(すぐ返る)。ループはせず、WAV自体を長く作って渡す(`Player::play`参照)。止めるのは[`stop_all`]。
pub fn play_loop(wav: &[u8]) -> bool {
    start(wav)
}

pub fn stop_all() {
    if let Some(mut c) = CURRENT.lock().unwrap_or_else(|e| e.into_inner()).take() {
        let _ = c.child.kill();
        let _ = c.child.wait();
        let _ = std::fs::remove_file(&c.file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_players_for_this_os() {
        assert!(!players().is_empty());
        assert!(players().iter().all(|(p, _)| !p.is_empty()));
    }

    #[test]
    fn stop_all_without_playback_is_harmless() {
        stop_all();
        stop_all();
    }
}
