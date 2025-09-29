// #[cfg(feature = "cc")]
pub mod CC100 {
    use std::fs::File;
    use std::io::{self, BufRead, BufReader};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use anyhow::{Context, Result};
    use arrow::array::{StringBuilder, UInt64Builder};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::arrow_writer::ArrowWriter;
    use parquet::basic::Compression;
    use parquet::file::properties::WriterProperties;

    struct ParquetRotator {
        base_out: PathBuf,
        part_idx: usize,
        max_rows_per_file: usize,
        max_row_group_rows: usize,
        writer: Option<ArrowWriter<File>>,
        rows_in_current_file: usize,
        schema: Arc<Schema>,
    }

    impl ParquetRotator {
        fn new(
            base_out: impl AsRef<Path>,
            schema: Arc<Schema>,
            max_rows_per_file: usize,
            max_row_group_rows: usize,
        ) -> Result<Self> {
            Ok(Self {
                base_out: base_out.as_ref().to_path_buf(),
                part_idx: 0,
                max_rows_per_file,
                max_row_group_rows,
                writer: None,
                rows_in_current_file: 0,
                schema,
            })
        }

        fn open_next(&mut self) -> Result<()> {
            if let Some(mut w) = self.writer.take() {
                w.close().context("failed to close parquet file")?;
            }
            let filename = self.base_out.with_extension("").with_file_name(format!(
                "{}.part{:05}.parquet",
                self.base_out.file_name().unwrap().to_string_lossy(),
                self.part_idx
            ));
            self.part_idx += 1;

            let file = File::create(&filename).with_context(|| format!("create {:?}", filename))?;

            // Parquet WriterProperties（圧縮や行グループ行数など）
            let props = WriterProperties::builder()
                .set_compression(Compression::ZSTD) // バランス良い既定候補。:contentReference[oaicite:10]{index=10}
                .set_max_row_group_size(self.max_row_group_rows) // 行グループを行数で制御。:contentReference[oaicite:11]{index=11}
                .build();

            let writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))
                .context("init ArrowWriter")?;
            self.writer = Some(writer);
            self.rows_in_current_file = 0;
            Ok(())
        }

        fn write_batch(&mut self, batch: &RecordBatch) -> Result<()> {
            if self.writer.is_none() {
                self.open_next()?;
            }
            let w = self.writer.as_mut().unwrap();
            w.write(batch).context("write record batch")?;
            self.rows_in_current_file += batch.num_rows();

            if self.rows_in_current_file >= self.max_rows_per_file {
                // ローテーション：Row Groupは内部で締まるので、そのままclose→open。
                self.open_next()?;
            }
            Ok(())
        }

        fn close(mut self) -> Result<()> {
            if let Some(mut w) = self.writer.take() {
                w.close().context("close parquet writer")?;
            }
            Ok(())
        }
    }

    /// CC-100 形式の巨大TXTを空行でドキュメント分割しつつ、ストリームでParquetへ。
    /// - txtの1塊 => 1レコード（{doc_id, text}）
    /// - メモリは小バッチのみ確保
    pub fn txt_to_parquet_stream(
        input_txt: &str,
        output_base: &str,
        batch_rows: usize,         // 例: 50_000
        max_row_group_rows: usize, // 例: 500_000 （平均文字量から128〜512MB相当に調整）
        max_rows_per_file: usize,  // 例: 5_000_000（≈ 数百MB〜1GB 目安）
    ) -> Result<()> {
        // Arrow スキーマ
        let schema = Arc::new(Schema::new(vec![
            Field::new("doc_id", DataType::UInt64, false),
            Field::new("text", DataType::Utf8, false),
        ]));

        // 入力
        let file = File::open(input_txt).with_context(|| format!("open input: {}", input_txt))?;
        let mut reader = BufReader::with_capacity(1 << 20, file); // 1MiB

        // 出力ローテータ
        let mut rot = ParquetRotator::new(
            output_base,
            schema.clone(),
            max_rows_per_file,
            max_row_group_rows,
        )?;

        // バッチ用ビルダ
        let mut id_builder = UInt64Builder::new();
        let mut text_builder = StringBuilder::new();

        // 行を積んで空行でドキュメント化
        let mut line_buf = String::new();
        let mut doc_buf = String::new();
        let mut next_id: u64 = 0;

        // 平均文字長のローリング推定（Row Group/ファイル行数チューニングの参考にしたい場合）
        let mut total_chars: u64 = 0;
        let mut total_docs: u64 = 0;

        loop {
            line_buf.clear();
            let n = reader
                .read_line(&mut line_buf)
                .context("read_line failed")?;
            if n == 0 {
                // EOF：最後のドキュメントがあれば吐く
                if !doc_buf.is_empty() {
                    id_builder.append_value(next_id);
                    text_builder.append_value(doc_buf.as_str());
                    next_id += 1;
                    total_chars += doc_buf.len() as u64;
                    total_docs += 1;
                    doc_buf.clear();
                }
                // 最終フラッシュ
                flush_batch(
                    &mut rot,
                    &mut id_builder,
                    &mut text_builder,
                    &schema,
                    batch_rows,
                )?;
                break;
            }

            let trimmed = line_buf.trim_end_matches(&['\r', '\n'][..]);

            if trimmed.is_empty() {
                // ドキュメント確定
                if !doc_buf.is_empty() {
                    id_builder.append_value(next_id);
                    text_builder.append_value(doc_buf.as_str());
                    next_id += 1;
                    total_chars += doc_buf.len() as u64;
                    total_docs += 1;
                    doc_buf.clear();

                    // バッチ境界：必要ならフラッシュ
                    if id_builder.len() >= batch_rows {
                        flush_batch(
                            &mut rot,
                            &mut id_builder,
                            &mut text_builder,
                            &schema,
                            batch_rows,
                        )?;
                    }
                }
            } else {
                if !doc_buf.is_empty() {
                    doc_buf.push('\n');
                }
                doc_buf.push_str(trimmed);
            }
        }

        rot.close()?;

        if total_docs > 0 {
            let avg = total_chars as f64 / total_docs as f64;
            eprintln!("avg chars per doc ≈ {:.1}", avg);
            eprintln!("wrote {} docs total", total_docs);
        }
        Ok(())
    }

    fn flush_batch(
        rot: &mut ParquetRotator,
        id_builder: &mut UInt64Builder,
        text_builder: &mut StringBuilder,
        schema: &Arc<Schema>,
        _batch_rows: usize,
    ) -> Result<()> {
        if id_builder.len() == 0 {
            return Ok(());
        }

        let id_array = Arc::new(id_builder.finish());
        let text_array = Arc::new(text_builder.finish());

        let batch = RecordBatch::try_new(schema.clone(), vec![id_array, text_array])
            .context("make record batch")?;

        rot.write_batch(&batch)?;
        Ok(())
    }

    fn main(cc_path: &str, output_path: &str) -> Result<()> {
        // 目安設定：
        // - BATCH_ROWS は小さめ（1〜5万）でOK（メモリ天井を下げる）
        // - MAX_ROW_GROUP_ROWS は平均文字数×行数 ≈ 128〜512MB を狙って調整
        // - MAX_ROWS_PER_FILE は数百万行（≒ 数百MB〜1GB）でローテーション
        txt_to_parquet_stream(
            cc_path,
            output_path,
            50_000,
            500_000,
            5_000_000,
        )
    }
}
