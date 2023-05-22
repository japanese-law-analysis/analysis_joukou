use anyhow::Result;
use clap::Parser;
use listup_law::LawData;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;
use tracing::*;

mod get_abbreviation;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// 解析結果を出力するJSONファイルへのpath
  #[clap(short, long)]
  output: String,
  /// エラーが出た条文の情報を出力するJSONファイルへのpath
  #[clap(short, long)]
  error_output: String,
  /// 略称一覧を出力するJSONファイルへのpath
  #[clap(short, long)]
  abbreviation_output: String,
  /// 法令XMLファイル群が置かれている作業ディレクトリへのpath
  #[clap(short, long)]
  work: String,
  /// 法令ファイルのインデックス情報が書かれたJSONファイルへのpath
  #[clap(short, long)]
  index_file: String,
}

async fn init_logger() -> Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();

  init_logger().await?;

  let mut index = File::open(&args.index_file).await?;
  let mut index_buf = Vec::new();
  index.read_to_end(&mut index_buf).await?;
  let index_str = String::from_utf8(index_buf)?;
  let index_data = serde_json::from_str::<Vec<LawData>>(&index_str)?;

  let mut index_stream = tokio_stream::iter(index_data);

  let work_dir_path = Path::new(&args.work);

  let mut abbreviation_lst = Vec::new();

  while let Some(law_data) = index_stream.next().await {
    info!("[START] get abbreviation: {}", law_data.name);
    let path = law_data.file;
    let file_path = &work_dir_path.join(path);
    let mut xml_file = File::open(&file_path).await?;
    let mut xml_text = Vec::new();
    xml_file.read_to_end(&mut xml_text).await?;
    let mut l = get_abbreviation::get_law_all_abbreviation(&law_data.num, &xml_text).await?;
    abbreviation_lst.append(&mut l);
    info!("[END] get abbreviation: {}", law_data.name);
  }

  info!("abbreviation_lst.len() = {}", abbreviation_lst.len());

  info!(
    "[START] write abbreviation to file: {}",
    &args.abbreviation_output
  );
  let mut abbreviation_output = File::create(&args.abbreviation_output).await?;
  let abbreviation_lst_str = serde_json::to_string_pretty(&abbreviation_lst)?;
  abbreviation_output
    .write_all(abbreviation_lst_str.as_bytes())
    .await?;
  abbreviation_output.write_all(b"\n").await?;
  info!(
    "[END] write abbreviation to file: {}",
    &args.abbreviation_output
  );

  Ok(())
}
