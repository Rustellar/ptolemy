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

        // 1行読み込み
        let read_bytes = reader.read_line(&mut documents).unwrap();
        // トリムを行う
        documents = documents.trim().to_string();
        
        if documents.contains(" ") {
            documents = documents.replace(" ", "。");
        }

        println!("ドキュメント: {:?}", documents);

        // Stringのクリア
        documents.clear();

        // さらに1行読み込み
        let read_bytes = reader.read_line(&mut documents).unwrap();
        // トリムを行う
        documents = documents.trim().to_string();

        if documents.contains(" ") {
            documents = documents.replace(" ", "。");
        }
        
        println!("ドキュメント: {:?}", documents);
    }
}
