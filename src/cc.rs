// #[cfg(feature = "cc")]
pub mod cc100 {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use indicatif::{ProgressBar, ProgressStyle};

    pub fn test(txt_path: &str, output_path: &str) {
        // envを読み込み
        let env_load = dotenvy::dotenv();
        let mut _input = String::new();
        let mut _output = String::new();

        // .envが存在する場合
        if env_load.is_ok() {
            _input = std::env::var("CC100_JA_INPUT").expect("CC100_JA_INPUTが設定されていません");
            _output = std::env::var("CC100_JA_OUTPUT").expect("CC100_JA_OUTPUTが設定されていません");
        } else {
            // .envが存在しない場合、デフォルト値を使用
            _input = txt_path.to_string();
            _output = output_path.to_string();
        }

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
        let input_file = File::open(_input).expect("ファイルが開けません");
        let mut reader = BufReader::new(input_file);
        let mut csv_writer = csv::Writer::from_writer(File::create(format!("{}/ja_{}.csv", _output, sentence_index)).expect("ファイルが作成できません"));

        // csvのヘッダーを書き込み
        csv_writer.write_record(&["index", "line"]).expect("CSVのヘッダーの書き込みに失敗しました");
        csv_writer.flush().expect("CSVのフラッシュに失敗しました");

        // 文書バッファ
        let mut documents = String::new();

        let mut index = 0;

        loop {
            bar.tick();

            // 1行読み込み
            let read_bytes = reader.read_line(&mut documents).unwrap();

            /*
            println!("\n----- Debug -----");
            println!("読み取ったバイト数: {}", read_bytes);
            println!("読み取った行: {}", documents);
            println!("行は空? : {}", documents.is_empty());
            println!("行は空白? : {}", documents.chars().all(char::is_whitespace));
            println!("-----------------\n");
            */

            // EOFに到達した場合
            if read_bytes == 0 {
                break;
            }

            // 空白行かどうかをチェック
            if documents.chars().all(char::is_whitespace) {
                documents.clear();

                // 次のファイルを作成
                csv_writer = csv::Writer::from_writer(File::create(format!("{}/ja_{}.csv", _output, sentence_index)).expect("ファイルが作成できません"));
                // csvのヘッダーを書き込み
                csv_writer.write_record(&["index", "line"]).expect("CSVのヘッダーの書き込みに失敗しました");
                csv_writer.flush().expect("CSVのフラッシュに失敗しました");

                // Sentence indexをインクリメント
                sentence_index += 1;
                // indexをリセット
                index = 0;
                continue;
            }

            // トリムを行う
            documents = documents.trim().to_string();

            if documents.contains(" ") {
                documents = documents.replace(" ", "。");
            }

            // indexとlineでcsvを作成
            csv_writer.write_record(&[format!("{}", index), format!("{}", documents)]).expect("CSVの書き込みに失敗しました");
            csv_writer.flush().expect("CSVのフラッシュに失敗しました");

            // indexをインクリメント
            index += 1;

            // Stringのクリア
            documents.clear();
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::{fs::File, io::{BufRead, BufReader}};

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
