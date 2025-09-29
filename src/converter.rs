// ルビを扱うモジュール
#[cfg(feature = "convert")]
pub mod RubyModule {
    use regex::Regex;

    // ルビ → [基]{読み} 変換(2パターン対応)
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
}

// 文字コードからUTF-8へ変換するモジュール
#[cfg(feature = "convert")]
pub mod UnicodeModule {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use encoding_rs_io::DecodeReaderBytesBuilder;

    /// # Convert to UTF-8
    /// 指定されたエンコードのファイルをUTF-8に変換して保存します。
    ///
    /// ## Example
    /// ```
    /// let file_path = "example_shit_jis.txt";
    /// let encode = UTF_16LE;
    /// convert_to_utf8(file_path, encode);
    /// ```
    pub fn convert_to_utf8(
        input_file: &str,
        encode: &'static encoding_rs::Encoding,
    ) -> Vec<String> {
        // デコーダーを介してUTF-8に変換
        let target_file = File::open(input_file).expect("ファイルが読み込めませんでした!");
        let decoder = DecodeReaderBytesBuilder::new()
            .encoding(Some(&encode))
            .build(BufReader::new(target_file));

        let mut reader = BufReader::new(decoder);

        // 1行ずつ読み込んで書き出し
        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        while reader.read_line(&mut line).unwrap() > 0 {
            lines.push(line);
            line = String::new();
        }
        lines
    }
}
