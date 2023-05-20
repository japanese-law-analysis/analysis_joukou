use anyhow::Result;
use clap::Parser;

mod article;
mod get_ryakusyou;



#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
  /// 解析結果を出力するJSONファイルへのpath
  #[clap(short, long)]
  output: String,
  /// エラーが出た条文の情報を出力するJSONファイルへのpath
  #[clap(short, long)]
  error_output: String,
  /// 法令XMLファイル群が置かれている作業ディレクトリへのpath
  #[clap(short, long)]
  work: String,
  /// 法令ファイルのインデックス情報が書かれたJSONファイルへのpath
  #[clap(short, long)]
  index_file: String,
  /// 解析する対象の条文のインデックスが書かれたJSONファイルへのpath
  #[clap(short, long)]
  article_info_file: String,
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

  init_logger().await?;

  println!("Hello, world!");
  Ok(())
}
