use anyhow::Result;
use jplaw_text::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

/// 法令名の略称
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Abbreviation {
  /// 法令番号
  num: String,
  /// 法令の略称
  name: String,
  /// 「本項において」などの条件文
  note: Option<String>,
  /// 記述されている条番号
  article: Article,
  law_num: String,
}

/// 法令名の略称を、その定義文から自動で抽出する関数
pub fn get_law_abbreviation(law_num: &str, article: &Article, text: &str) -> Vec<Abbreviation> {
  let mut lst = Vec::new();
  let re = Regex::new(
    r#"（(?P<num>(明治|大正|昭和|平成|令和)(一|二|三|四|五|六|七|八|九|十|〇)+年[^（）、。あ-ん]+第(一|二|三|四|五|六|七|八|九|十|百|千|〇)+号).+(以下)?((?P<note>[^」）]+)において)?、?「(?P<name>[^」]*(法|令))」という。"#,
  )
  .unwrap();
  for caps in re.captures_iter(text) {
    let num_opt = caps.name("num").map(|n| n.as_str().to_string());
    let name_opt = caps.name("name").map(|n| n.as_str().to_string());
    let note_match = caps.name("note");
    let note = note_match.map(|n| n.as_str().to_string());
    if let (Some(num), Some(name)) = (num_opt, name_opt) {
      lst.push(Abbreviation {
        num,
        name,
        note,
        article: article.clone(),
        law_num: law_num.to_string(),
      })
    }
  }
  lst
}

pub async fn get_law_all_abbreviation(law_num: &str, law_xml: &[u8]) -> Result<Vec<Abbreviation>> {
  let text_lst = xml_to_law_text(law_xml).await?;
  let mut lst = Vec::new();
  let mut text_stream = tokio_stream::iter(text_lst);
  while let Some(law_text) = text_stream.next().await {
    let contents = law_text.contents;
    match contents {
      LawContents::Text(text) => {
        let mut l = get_law_abbreviation(law_num, &law_text.article_info, &text);
        lst.append(&mut l);
      }
      LawContents::Table(_) => {
        // 表中で定義されることは無いので何もしない
      }
    }
  }
  Ok(lst)
}
