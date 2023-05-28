use anyhow::Result;
use clap::Parser;
use listup_law::LawData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::StreamExt;
use tracing::*;

mod get_abbreviation;
mod get_law_name;

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
  abbreviation_output: Option<String>,
  /// 法令XMLファイル群が置かれている作業ディレクトリへのpath
  #[clap(long)]
  work: String,
  /// 法令ファイルのインデックス情報が書かれたJSONファイルへのpath
  #[clap(long)]
  index_file: String,
  /// 法令ファイルのインデックス情報が書かれたJSONファイルへのpath
  #[clap(long)]
  abb_list: String,
}

async fn init_logger() -> Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Abb {
  /// 法令番号
  num: String,
  /// 略称
  abbs: Vec<String>,
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

  let mut index_stream = tokio_stream::iter(index_data.clone());

  let work_dir_path = Path::new(&args.work);

  let mut abbreviation_lst: HashMap<String, Vec<get_abbreviation::Abbreviation>> = HashMap::new();

  let mut law_name_map: HashMap<String, Vec<String>> = HashMap::new();

  while let Some(law_data) = index_stream.next().await {
    law_name_map.insert(law_data.clone().num, vec![law_data.clone().name]);
    info!("[START] get abbreviation: {}", law_data.name);
    let path = law_data.file;
    let file_path = &work_dir_path.join(path);
    let mut xml_file = File::open(&file_path).await?;
    let mut xml_text = Vec::new();
    xml_file.read_to_end(&mut xml_text).await?;
    let l = get_abbreviation::get_law_all_abbreviation(&xml_text).await?;
    if !l.is_empty() {
      abbreviation_lst.insert(law_data.num, l);
    };
    info!("[END] get abbreviation: {}", law_data.name);
  }

  if let Some(abbreviation_path) = &args.abbreviation_output {
    info!("[START] write abbreviation to file: {}", abbreviation_path);
    let mut abbreviation_output = File::create(abbreviation_path).await?;
    let abbreviation_lst_str = serde_json::to_string_pretty(&abbreviation_lst)?;
    abbreviation_output
      .write_all(abbreviation_lst_str.as_bytes())
      .await?;
    abbreviation_output.write_all(b"\n").await?;
    info!("[END] write abbreviation to file: {}", abbreviation_path);
  }

  let mut egov_abb_file = File::open(&args.abb_list).await?;
  let mut egov_abb_buf = Vec::new();
  egov_abb_file.read_to_end(&mut egov_abb_buf).await?;
  let egov_abb_str = String::from_utf8(egov_abb_buf)?;
  let egov_abb_lst = serde_json::from_str::<Vec<Abb>>(&egov_abb_str)?;
  let mut egov_abb_stream = tokio_stream::iter(egov_abb_lst);
  while let Some(egov_abb) = egov_abb_stream.next().await {
    if let Some(lst) = law_name_map.get(&egov_abb.num) {
      let mut l = egov_abb.clone().abbs;
      l.push(lst[0].clone()); // lstの中身は正式名称の1つしか入っていないのでこれでよい
      law_name_map.insert(egov_abb.clone().num, lst.clone());
    } else {
      law_name_map.insert(egov_abb.clone().num, egov_abb.clone().abbs);
    }
  }
  let mut index_stream = tokio_stream::iter(index_data.clone());
  while let Some(law_data) = index_stream.next().await {
    info!("[START] get law name: {}", law_data.name);
    let path = law_data.file;
    let file_path = &work_dir_path.join(path);
    let mut xml_file = File::open(&file_path).await?;
    let mut xml_text = Vec::new();
    xml_file.read_to_end(&mut xml_text).await?;
    // TODO
    info!("[END] get law name: {}", law_data.name);
  }

  Ok(())
}
