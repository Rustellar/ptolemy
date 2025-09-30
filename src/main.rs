use std::{
    fs::{self, File},
    io::{BufRead, BufReader}, path::PathBuf,
};

use indicatif::{ProgressBar, ProgressStyle};

use csv::Writer;

pub mod cc;

fn main() {
    cc::cc100::test();

    // プログレスバー関連
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::with_template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"])
        );
        bar.set_message("まとめ中...");

    // csvを1GiB程度にまとめる
    let csv_path = "/home/ok240108@OSAKA-NET.nkz.ac.jp/mnt/file_server/ok24/OK240108/data/cc100/ja/";
    let csv_path = PathBuf::new().join(csv_path);

    // フォルダか?
    if csv_path.is_dir() {
        // フォルダ内のCSVファイルを取得
        let csv_files = csv_path.read_dir().expect("フォルダの読み込みに失敗しました");
        
        // フォルダ内のcsvファイル名を出力
        for file in csv_files {
            bar.tick();

            let file = file.expect("ファイルの読み込みに失敗しました");
            let file_path = file.path();

            // 統合するcsvファイルのVector
            let mut combined_csv: Vec<PathBuf> = Vec::new();
            // 今までのファイルサイズ
            let mut total_file_size: u64 = 0;
            // ファイルサイズの上限(1GiB)
            let max_file_size: u64 = 1 * 1024 * 1024 * 1024;

            // 拡張子がcsvか?
            if let Some(ext) = file_path.extension() {
                if ext == "csv" {
                    // メタデータを取得
                    match fs::metadata(&file_path) {
                        Ok(metadata) => {
                            // ファイルサイズを取得
                            let file_size = metadata.len();
                            // ファイルサイズを合計し、上限を超えないか?
                            if 1024 + max_file_size >= total_file_size + file_size {
                                combined_csv.push(file_path.clone());
                                total_file_size += file_size;
                            } else {
                                // 上限を超える場合、現在のcombined_csvを出力し、新しいファイルを開始
                                if !combined_csv.is_empty() {
                                    // 統合ファイルを作成
                                    let output_file_path = csv_path.join(format!("combined_{}.csv", combined_csv.len()));
                                    // 合成したcsvのindex
                                    let mut combine_index = 0;
                                    // バッファー
                                    let mut buffer = String::new();

                                    let mut writer = Writer::from_writer(File::create(&output_file_path).expect("統合ファイルの作成に失敗しました"));
                                    // csvのヘッダーを書き込み
                                    writer.write_record(&["csv_id", "index", "line"]).expect("CSVのヘッダーの書き込みに失敗しました");
                                    writer.flush().expect("CSVのフラッシュに失敗しました");

                                    for (csv_index, csv_file) in combined_csv.iter().enumerate() {
                                        let input_file = File::open(csv_file).expect("入力ファイルのオープンに失敗しました");
                                        let reader = BufReader::new(input_file);

                                        for line in reader.lines() {
                                            let line = line.expect("行の読み込みに失敗しました");
                                            // ヘッダー行はスキップ
                                            if line.starts_with("index,line") {
                                                continue;
                                            }
                                            // indexとlineに分割
                                            let parts: Vec<&str> = line.splitn(2, ',').collect();
                                            let parts = parts.get(0).unwrap_or(&"");
                                            // 統合ファイルに書き込み
                                            writer.write_record(&[format!("{}", csv_index), format!("{}", combine_index), parts.to_string()]).expect("統合ファイルの書き込みに失敗しました");
                                            combine_index += 1;
                                        }
                                        writer.flush().expect("統合ファイルのフラッシュに失敗しました");
                                        // バッファーをクリア
                                        buffer.clear();
                                    }
                                }
                                // 新しいファイルを開始
                                combined_csv = vec![file_path.clone()];
                                total_file_size = file_size;
                            }
                        }
                        Err(e) => {
                            eprintln!("メタデータの取得に失敗しました: {:?}", e);
                        }
                    }
                }
            }
        }
    } else {
        println!("指定されたパスはフォルダではありません: {:?}", csv_path);
    }
}
