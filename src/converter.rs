use regex::Regex;

// ルビ → [基]{読み} 変換（2パターン対応）
pub fn convert_ruby(s: &str) -> String {
    // <ruby>漢字<rt>よみかた</rt></ruby>
    let pattern = Regex::new(
        r#"(?s)<ruby>\s*(?P<base>[^<]*?)\s*<rt>\s*(?P<read>[^<]*?)\s*</rt>\s*</ruby>"#,
    )
    .unwrap();

    // 変換
    let converted = pattern.replace_all(s, |caps: &regex::Captures| {
        format!("[{}]{{{}}}", caps["base"].trim(), caps["read"].trim())
    });

    converted.into_owned()
}
