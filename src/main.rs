use std::{fs::File, io::{BufRead, BufReader, Read}};

use indicatif::{ProgressBar, ProgressIterator};

use csv::Writer;

pub mod cc;

fn main() {
    let input = "/home/ok240108@OSAKA-NET.nkz.ac.jp/mnt/file_server/ok24/OK240108/data/cc100/ja.txt";
    let output = "/home/ok240108@OSAKA-NET.nkz.ac.jp/mnt/file_server/ok24/OK240108/data/cc100/ja";

    let input_file = File::open(input).expect("ファイルが開けません");
    let mut reader = BufReader::new(input_file);
    let mut suppress_line = String::new();

    // 退避用のバッファ
    let mut buffer = String::new();

    // 文書バッファ
    let mut documents: Vec<String> = Vec::new();

    let bar = ProgressBar::no_length();

    // 文字塊のインデックス
    let mut chunk_index = 1;
        
    // ここを改良する必要あり
    for (index, line) in reader.lines().enumerate() {
        let line = line.expect("行の読み込みに失敗しました");

        if index >= 200 {
            break;
        }

        bar.inc(1);

        // 空行であれば文書の区切りとみなす
        if line.trim().is_empty() {
            // バッファをVecに書き込み
            if !buffer.trim().is_empty() {
                documents.push(buffer.trim().to_string());

                // csvに書き込み
                let output = File::create(format!("{}_chunk_{}.csv", output, chunk_index)).expect("ファイルの作成に失敗しました");
                let mut writter = csv::Writer::from_writer(&output);
                // レコード書き込み
                writter.write_record(&["index", "text"]).expect("ヘッダーの書き込みに失敗しました");
                // データを書き込む
                writter.write_record(&[index.to_string()]).expect("書き込みに失敗しました");
                writter.write_record(&[documents.join("\n")]).expect("書き込みに失敗しました");
                writter.flush().expect("フラッシュに失敗しました");
                buffer.clear();
                chunk_index += 1;
            }
        }
        else {
            // バッファに行を追加
            buffer.push_str(&line);
            buffer.push('\n');
        }
    }

    bar.finish();

    println!("ドキュメント: {:?}", documents);
}