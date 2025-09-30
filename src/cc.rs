// #[cfg(feature = "cc")]
pub mod cc100 {}

#[cfg(test)]
pub mod tests {

    use std::{
        fs::File,
        io::{BufRead, BufReader, Read},
    };

    use indicatif::{ProgressBar, ProgressStyle};

    use super::*;

    #[test]
    pub fn test() {
        let input =
            "/home/ok240108@OSAKA-NET.nkz.ac.jp/mnt/file_server/ok24/OK240108/data/cc100/ja.txt";
        let output = "/home/ok240108@OSAKA-NET.nkz.ac.jp/mnt/file_server/ok24/OK240108/data/cc100/ja";
        
        // 文の塊毎にindexを更新する
        let mut sentence_index = 1;

        // プログレスバー関連
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        );
        bar.set_message("processing...");

        // io関連
        let input_file = File::open(input).expect("ファイルが開けません");
        let mut reader = BufReader::new(input_file);
        let mut writer = File::create(format!("{}/ja_{}.txt", output, sentence_index)).expect("ファイルが作成できません");

        // 文書バッファ
        let mut documents = String::new();

        // デバッグ用
        let mut _debug_index = 0;

        loop {
            bar.tick();

            // デバッグ用
            if _debug_index > 100 {
                break;
            }

            _debug_index += 1;

            // 1行読み込み
            let read_bytes = reader.read_line(&mut documents).unwrap();

            println!("\n----- Debug -----");
            println!("読み取ったバイト数: {}", read_bytes);
            println!("読み取った行: {}", documents);
            println!("行は空? : {}", documents.is_empty());
            println!("行は空白? : {}", documents.chars().all(char::is_whitespace));
            println!("-----------------\n");

            // 読み取ったバイトが0?
            if documents.chars().all(char::is_whitespace) || read_bytes == 0 {
                // 次の行を読み取り、Noneだった場合はEOFと判断
                let mut _buf = String::new();
                if let Err(_) = reader.read_line(&mut _buf) {
                    println!("EOFに到達しました");
                    break;
                }
                // EOFじゃない場合は、分割を行う
                else {
                    println!("-----分割-----");
                }
            }
            else {
                // トリムを行う
                documents = documents.trim().to_string();
            
                if documents.contains(" ") {
                    documents = documents.replace(" ", "。");
                }

                println!("ドキュメント: {}", documents);

                // Stringのクリア
                documents.clear();
            }
        }
    }

    #[test]
    pub fn print() {
        let input =
            "/home/ok240108@OSAKA-NET.nkz.ac.jp/mnt/file_server/ok24/OK240108/data/cc100/ja.txt";
        let input_file = File::open(input).expect("ファイルが開けません");
        let reader = BufReader::new(input_file);

        for (index, line) in reader.lines().enumerate() {
            if index > 100 {
                break;
            }

            let line = line.expect("ファイルが読み込めません");
            println!("{}: {}", index + 1, line);
        }
    }
}
