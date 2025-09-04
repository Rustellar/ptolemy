use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
};

use clap::Parser;
use encoding_rs::UTF_16LE;
use encoding_rs_io::DecodeReaderBytesBuilder;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    // 処理を行うファイルのパス
    input_file_path: String,
    // 処理後に保存するファイルのパス
    output_file_path: String,
}

// UTF-16LEを想定
pub fn convert_to_utf8(file_path: &str) {
    // デコーダーを介してUTF-8に変換
    let input_file = File::open(file_path).expect("ファイルが読み込めませんでした!");
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(UTF_16LE))
        .build(BufReader::new(input_file));

    let mut reader = BufReader::new(decoder);

    // 出力ファイルの作成
    // 拡張子で分割
    let output_path: Vec<&str> = file_path.split(".").collect();
    let output_path = format!("{}_utf8.{}", output_path.get(0).unwrap(), output_path.get(1).unwrap());
    let output_file = File::create(output_path).expect("ファイルの作成に失敗しました!");
    let mut writter = BufWriter::new(output_file);

    // 1行ずつ読み込んで書き出し
    let mut line = String::new();
    while reader.read_line(&mut line).unwrap() > 0 {
        writter.write_all(line.as_bytes()).expect("書き込み中にエラーが発生しました!");
        line.clear();
    }

    writter.flush().expect("フラッシュ中にエラーが発生しました!");
}
