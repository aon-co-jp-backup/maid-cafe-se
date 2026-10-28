//! Windows以外の音声合成。macOS=`say`(標準搭載)、Linux=`espeak-ng`(要インストール)。どちらも文章をファイルで渡し、WAVへ書き出させる
//! (その後の声の加工〜再生は、Windowsと同じ流れ)。コマンドの組み立てと出力の解析は純粋関数にしてあり、どのOSでもテストできる。
//!
//! 実機の確認状況: macOS/Linuxの実機では未確認(開発機がWindowsのため)。CI(GitHub Actions)で、コマンドが使える場合だけ実際に合成するテストを走らせる。

use crate::sapi::{SapiVoice, Synthesized};
use maid_cafe_core::audio::SourceGender;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// `say`/`espeak-ng`の標準の話速(WPM)。
pub const BASE_WPM: f64 = 175.0;

/// SAPI相当のRate(-10..10、約1/3倍〜3倍)を、話速(WPM)へ。
pub fn wpm(rate_step: i32) -> u32 {
    (BASE_WPM * 3f64.powf(rate_step.clamp(-10, 10) as f64 / 10.0)).round().clamp(80.0, 450.0) as u32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Engine {
    /// macOSの`say`
    Say,
    /// Linuxの`espeak-ng`(古い環境の`espeak`も)
    Espeak,
}

fn work_dir() -> std::io::Result<PathBuf> {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!("maid-cafe-se-tts-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 言語コードを`ja-JP`の形にそろえる(`ja_JP`・`en-us`・`ja`→それぞれ`ja-JP`・`en-US`・`ja`)。
pub fn normalize_culture(s: &str) -> String {
    let s = s.trim().replace('_', "-");
    match s.split_once('-') {
        Some((l, r)) => format!("{}-{}", l.to_ascii_lowercase(), r.to_ascii_uppercase()),
        None => s.to_ascii_lowercase(),
    }
}

fn culture_matches(culture: &str, lang: &str) -> bool {
    let c = culture.to_ascii_lowercase();
    c == lang.to_ascii_lowercase() || c.starts_with(&format!("{}-", lang.to_ascii_lowercase()))
}

/// `say -v ?`の出力を解析する。1行は`名前(空白あり得る)  ja_JP    # サンプル文`の形。
pub fn parse_say_voices(out: &str) -> Vec<SapiVoice> {
    out.lines()
        .filter_map(|l| {
            let head = l.split('#').next()?.trim_end();
            let (name, loc) = head.rsplit_once(char::is_whitespace)?;
            let name = name.trim();
            (!name.is_empty() && loc.contains('_')).then(|| SapiVoice { name: name.into(), culture: normalize_culture(loc), gender: say_gender(name) })
        })
        .collect()
}

/// `say`は性別を教えてくれないので、日本語で知られている声だけ対応させる(他は不明=声の加工側で自動判断)。
fn say_gender(name: &str) -> SourceGender {
    match name.split(' ').next().unwrap_or("") {
        "Kyoko" | "O-Ren" => SourceGender::Female,
        "Otoya" | "Hattori" => SourceGender::Male,
        _ => SourceGender::Unknown,
    }
}

/// `espeak-ng --voices`の出力を解析する。列は`Pty Language Age/Gender VoiceName File Other`。声の名前には言語コードを使う。
pub fn parse_espeak_voices(out: &str) -> Vec<SapiVoice> {
    out.lines()
        .skip(1)
        .filter_map(|l| {
            let c: Vec<&str> = l.split_whitespace().collect();
            if c.len() < 4 {
                return None;
            }
            let gender = if c[2].ends_with("/F") {
                SourceGender::Female
            } else if c[2].ends_with("/M") {
                SourceGender::Male
            } else {
                SourceGender::Unknown
            };
            Some(SapiVoice { name: c[1].into(), culture: normalize_culture(c[1]), gender })
        })
        .collect()
}

/// 合成コマンドの引数(プログラム名は含まない)。文章はファイル`input`、出力は`out`(WAV)。
pub fn synth_args(engine: Engine, voice: Option<&str>, wpm: u32, input: &Path, out: &Path) -> Vec<String> {
    let mut a: Vec<String> = Vec::new();
    match engine {
        Engine::Say => {
            if let Some(v) = voice {
                a.extend(["-v".into(), v.into()]);
            }
            a.extend(["-r".into(), wpm.to_string(), "-f".into(), input.display().to_string()]);
            a.extend(["-o".into(), out.display().to_string(), "--file-format=WAVE".into(), "--data-format=LEI16@22050".into()]);
        }
        Engine::Espeak => {
            if let Some(v) = voice {
                a.extend(["-v".into(), v.into()]);
            }
            a.extend(["-s".into(), wpm.to_string(), "-f".into(), input.display().to_string(), "-w".into(), out.display().to_string()]);
        }
    }
    a
}

fn run(program: &str, args: &[String], timeout: Duration) -> Result<String, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{program}を起動できません: {e}"))?;
    let mut out = child.stdout.take().ok_or("stdoutを取れません")?;
    let mut err = child.stderr.take().ok_or("stderrを取れません")?;
    let t_out = std::thread::spawn(move || {
        let mut b = Vec::new();
        let _ = std::io::Read::read_to_end(&mut out, &mut b);
        b
    });
    let t_err = std::thread::spawn(move || {
        let mut b = Vec::new();
        let _ = std::io::Read::read_to_end(&mut err, &mut b);
        b
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("音声合成がタイムアウトしました".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(format!("{program}の待機に失敗: {e}")),
        }
    };
    let stdout = String::from_utf8_lossy(&t_out.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&t_err.join().unwrap_or_default()).into_owned();
    if !status.success() {
        return Err(format!("音声合成に失敗しました({program}): {}", stderr.trim()));
    }
    Ok(stdout)
}

/// このOSで使う合成エンジンと、そのコマンド名。見つからなければ`None`。
pub fn detect() -> Option<(Engine, &'static str)> {
    let candidates: &[(Engine, &str)] = if cfg!(target_os = "macos") { &[(Engine::Say, "say")] } else { &[(Engine::Espeak, "espeak-ng"), (Engine::Espeak, "espeak")] };
    candidates.iter().find(|(_, p)| run(p, &["--version".into()], Duration::from_secs(5)).is_ok() || *p == "say" && Path::new("/usr/bin/say").exists()).copied()
}

fn missing_message() -> String {
    if cfg!(target_os = "macos") {
        "macOSの`say`コマンドが見つかりません".into()
    } else {
        "読み上げには`espeak-ng`が必要です。`sudo apt install espeak-ng`(Debian/Ubuntu)等で入れてください".into()
    }
}

/// インストール済みの音声一覧。取得できなければ空。
pub fn list_voices() -> Vec<SapiVoice> {
    match detect() {
        Some((Engine::Say, p)) => run(p, &["-v".into(), "?".into()], Duration::from_secs(15)).map(|o| parse_say_voices(&o)).unwrap_or_default(),
        Some((Engine::Espeak, p)) => run(p, &["--voices".into()], Duration::from_secs(15)).map(|o| parse_espeak_voices(&o)).unwrap_or_default(),
        None => Vec::new(),
    }
}

/// 区切りごとにWAVへ合成する。声は`voice_name`、未指定ならその言語の最初の声(espeak-ngは言語コードそのもの)。
/// 「その言語の声が無い」ときのエラーには`no voice for`を含める(呼び出し側が利用者向けの文言にする)。
pub fn synthesize(items: &[(i32, String)], voice_name: Option<&str>, lang: &str) -> Result<Synthesized, String> {
    if items.is_empty() {
        return Err("読み上げる文章がありません".into());
    }
    let (engine, program) = detect().ok_or_else(missing_message)?;
    let voices = list_voices();
    let voice = match voice_name.filter(|n| !n.is_empty()) {
        Some(n) => voices.iter().find(|v| v.name == n).cloned().unwrap_or(SapiVoice { name: n.into(), culture: String::new(), gender: SourceGender::Unknown }),
        None => voices.iter().find(|v| culture_matches(&v.culture, lang)).cloned().ok_or_else(|| format!("no voice for {lang}"))?,
    };
    let dir = work_dir().map_err(|e| e.to_string())?;
    let result = (|| {
        let mut wavs = Vec::new();
        for (i, (rate, text)) in items.iter().enumerate() {
            let input = dir.join(format!("in_{i}.txt"));
            let out = dir.join(format!("seg_{i}.wav"));
            std::fs::write(&input, text.replace(['\r', '\n'], " ")).map_err(|e| e.to_string())?;
            run(program, &synth_args(engine, Some(&voice.name), wpm(*rate), &input, &out), Duration::from_secs(60))?;
            let bytes = std::fs::read(&out).map_err(|e| format!("WAVができていません(seg_{i}): {e}"))?;
            if bytes.len() < 100 {
                return Err(format!("WAVが空です(seg_{i})"));
            }
            wavs.push(bytes);
        }
        Ok(Synthesized { wavs, voice: voice.clone() })
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wpm_maps_rate_steps() {
        assert_eq!(175, wpm(0));
        assert!(wpm(-3) < 175 && wpm(3) > 175);
        assert_eq!(wpm(10), wpm(99));
        assert_eq!(wpm(-10), wpm(-99));
        assert!(wpm(10) <= 450 && wpm(-10) >= 80);
    }

    #[test]
    fn normalizes_cultures() {
        assert_eq!("ja-JP", normalize_culture("ja_JP"));
        assert_eq!("en-US", normalize_culture("en-us"));
        assert_eq!("ja", normalize_culture("JA"));
        assert!(culture_matches("ja-JP", "ja") && culture_matches("ja", "ja") && !culture_matches("jam-JM", "ja"));
    }

    #[test]
    fn parses_say_voice_list() {
        let out = "Albert              en_US    # Hello! My name is Albert.\n\
                   Kyoko               ja_JP    # こんにちは、私の名前はKyokoです。\n\
                   Eddy (日本語(日本))  ja_JP    # こんにちは、Eddyです。\n";
        let v = parse_say_voices(out);
        assert_eq!(3, v.len());
        assert_eq!(SapiVoice { name: "Kyoko".into(), culture: "ja-JP".into(), gender: SourceGender::Female }, v[1]);
        assert_eq!("Eddy (日本語(日本))", v[2].name);
        assert!(v[1].is_japanese() && !v[0].is_japanese());
    }

    #[test]
    fn parses_espeak_voice_list() {
        let out = "Pty Language       Age/Gender VoiceName          File                 Other Languages\n\
                    5  af              --/M      Afrikaans          gmw/af\n\
                    5  ja              --/M      Japanese           asia/ja\n\
                    5  en-us           --/M      English_(America)  en-us                (en 3)\n";
        let v = parse_espeak_voices(out);
        assert_eq!(3, v.len());
        assert_eq!(SapiVoice { name: "ja".into(), culture: "ja".into(), gender: SourceGender::Male }, v[1]);
        assert_eq!("en-US", v[2].culture);
    }

    #[test]
    fn builds_synth_args() {
        let a = synth_args(Engine::Say, Some("Kyoko"), 150, Path::new("in.txt"), Path::new("o.wav"));
        assert_eq!(["-v", "Kyoko", "-r", "150", "-f", "in.txt", "-o", "o.wav", "--file-format=WAVE", "--data-format=LEI16@22050"].map(String::from).to_vec(), a);
        let b = synth_args(Engine::Espeak, Some("ja"), 120, Path::new("in.txt"), Path::new("o.wav"));
        assert_eq!(["-v", "ja", "-s", "120", "-f", "in.txt", "-w", "o.wav"].map(String::from).to_vec(), b);
    }

    /// 実際にコマンドがあるときだけ(CIのmacOS/Linux)、合成してWAVになることを確かめる。日本語の声が無ければ英語で確かめる。
    #[cfg(not(windows))]
    #[test]
    fn real_engine_synthesizes_when_available() {
        if detect().is_none() {
            eprintln!("SKIP: say/espeak-ngが無い");
            return;
        }
        let voices = list_voices();
        let lang = if voices.iter().any(|v| v.is_japanese()) { "ja" } else { "en" };
        let text = if lang == "ja" { "こんにちは" } else { "hello" };
        let out = match synthesize(&[(0, text.into()), (-2, format!("{text} {text} {text}"))], None, lang) {
            Ok(o) => o,
            Err(e) if e.contains("no voice for") => {
                eprintln!("SKIP: 声が無い: {e}");
                return;
            }
            Err(e) => panic!("{e}"),
        };
        assert_eq!(2, out.wavs.len());
        for w in &out.wavs {
            assert_eq!(b"RIFF", &w[0..4]);
            assert!(maid_cafe_core::audio::parse_wav(w).is_some(), "parse_wavで読める");
        }
        assert!(out.wavs[1].len() > out.wavs[0].len());
    }
}
