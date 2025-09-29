// 仮名を取り扱うモジュール
#[cfg(feature = "kana")]
pub mod KanaModule {
    use regex::bytes;

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum CharType {
        // 半角大文字アルファベット
        HalfWidthBigAlphaBet,
        // 半角小文字アルファベット
        HalfWidthSmallAlphaBet,
        // 半角数字
        HalfWidthNumber,
        // 平仮名
        Hiragana,
        // 片仮名
        Katakana,
        // 漢字
        Kanji,
        // 長音記号(ー)
        JapaneseLongSymbol,
        // 句読点
        JapanesePunctuationMark,
        // 鍵括弧(「)
        JapaneseStartBracket,
        // 鍵括弧(」)
        JapaneseEndBracket,
        // 未実装
        Other,
    }

    #[derive(Debug, Clone, Copy)]
    struct CharInfo {
        char: char,
        char_type: CharType,
    }

    pub fn utf8_split(input: &str) -> Vec<String> {
        let char_infos = get_array(input);
        // 分割結果を保存する
        let mut result = Vec::<String>::new();

        // 直前のcharとタイプを保持する
        let mut before_type: CharType = char_infos.get(0).unwrap().char_type;
        // まとまりを保持する
        let mut group = String::new();

        for (i, data) in char_infos.iter().enumerate() {
            if i == 0 {
                group.push(data.char);
                continue;
            }

            // 直前の型と一致するか？
            if before_type.eq(&data.char_type) || data.char_type.eq(&CharType::JapaneseLongSymbol) {
                // 一致したならば、まとまりを継続
                group.push(data.char);
                // 直前の更新
                before_type = data.char_type;
            } else {
                // 一致していないため、分割
                result.push(group.clone());
                // まとまりを初期化
                group.clear();

                group.push(data.char);
                // 直前の更新
                before_type = data.char_type;
            }
        }

        result.push(group);

        return result;
    }

    // 判定処理
    fn get_array(input: &str) -> Vec<CharInfo> {
        let mut result = Vec::<CharInfo>::new();

        for c in input.chars() {
            // charからStringへ
            let str = c.to_string();
            // タイプ
            let str_type = get_char_type(str.as_bytes());

            result.push(CharInfo {
                char: c,
                char_type: str_type,
            });
        }

        return result;
    }

    // UTF-8バイト列を受け取り、そのタイプを判定する
    fn get_char_type(c: &[u8]) -> CharType {
        // バイト長を結合する
        let c = digit_up(c);

        match c {
            // 半角大文字英語の範囲はA[41]~Z[5A]
            0x41..=0x5a => CharType::HalfWidthBigAlphaBet,
            // 半角小文字英語の範囲はa[61]~z[7A]
            0x61..=0x7a => CharType::HalfWidthSmallAlphaBet,
            // 半角数字の範囲は0[30]~9[39]
            0x30..=0x39 => CharType::HalfWidthNumber,
            // 平仮名はぁ[E3 81 81]~ゖ[E3 82 96]と、濁点半濁点の[E3 82 99]~[E3 82 9c]
            0xe38181..=0xe38296 => CharType::Hiragana,
            // 片仮名はァ[E3 82 A1]~ヶ[E3 83 B6]
            0xe382a1..=0xe383b6 => CharType::Katakana,
            // 長音記号は[E3 83 BC]
            0xe383bc => CharType::JapaneseLongSymbol,
            // 句読点は[E3 80 81]と[E3 80 82]
            0xe38081..=0xe38082 => CharType::JapanesePunctuationMark,
            // 漢字の範囲は
            // [E4 B8 80]~[E9 BF AA]
            // [EF A4 80]~[EF A9 AD]
            // [EF A9 B0]~[EF A9 B2]
            // [EF A9 B4]~[EF A9 B8]
            // [EF A9 BA]~[EF AA 80]
            // [EF AA 82]~[EF AA AB]
            // [EF AA AD]~[EF AA B4]
            // [EF AA B6]~[EF AB 83]
            // [EF AB 85]~[EF AB 8A]
            // [EF AB 8C]~[EF AB 8E]
            // [EF AB 91]
            // [EF AB 94]
            // [EF AB 98]
            0xe4b880..=0xe9bfaa
            | 0xefa480..=0xefa9ad
            | 0xefa9b0..=0xefa9b2
            | 0xefa9b4..=0xefa9b8
            | 0xefa9ba..=0xefaa80
            | 0xefaa82..=0xefaaab
            | 0xefaaad..=0xefaab4
            | 0xefaab6..=0xefab83
            | 0xefab85..=0xefab8a
            | 0xefab8c..=0xefab8e
            | 0xefab91
            | 0xefab94
            | 0xefab98 => CharType::Kanji,
            // 鍵括弧(「)は[E3 80 8C]
            0xe3808c => CharType::JapaneseStartBracket,
            // 鍵括弧(」)は[E3 80 8D
            0xe3808d => CharType::JapaneseEndBracket,
            // その他
            _ => CharType::Other,
        }
    }

    // 平仮名から片仮名への変換
    pub fn kata_to_hira(s: &str) -> String {
        s.chars()
            .map(|ch| match ch {
                '\u{30A1}'..='\u{30F6}' => char::from_u32(ch as u32 - 0x60).unwrap(),
                _ => ch, // 長音「ー」や記号はそのまま
            })
            .collect()
    }

    pub fn hira_to_kata(s: &str) -> String {
        s.chars()
            .map(|ch| match ch {
                '\u{3041}'..='\u{3096}' => char::from_u32(ch as u32 + 0x60).unwrap(),
                _ => ch,
            })
            .collect()
    }

    fn digit_up(number: &[u8]) -> u64 {
        let mut result: u64 = 0;
        for num in number {
            // 1byte上げ、OR演算で結合
            result = (result << 8) | *num as u64;
        }

        return result;
    }

    // 片仮名かどうか判定
    pub fn is_katakana(s: &str) -> bool {
        let re = bytes::Regex::new(r"^[\u30A1-\u30F6ー]+$").unwrap();
        re.is_match(s.as_bytes())
    }

    // テスト
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_kata_to_hira() {
            let input = "カタカナ";
            let expected = "かたかな";
            let result = kata_to_hira(input);
            assert_eq!(result, expected);
            println!("変換前: {} -> 変換後: {}", input, result);
        }

        #[test]
        fn test_hira_to_kata() {
            let input = "ひらがな";
            let expected = "ヒラガナ";
            let result = hira_to_kata(input);
            assert_eq!(result, expected);
            println!("変換前: {} -> 変換後: {}", input, result);
        }
    }
}
