use regex::Regex;
use serde::{Deserialize, Serialize};

/// 法令名の略称
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Ryakusyou {
  /// 法令番号
  num: String,
  /// 法令の略称
  name: String,
  /// 「本項において」などの条件文
  note: Option<String>,
}

/// 法令名の略称を、その定義文から自動で抽出する関数
pub fn get_law_ryakusyou(text: &str) -> Vec<Ryakusyou> {
  let mut lst = Vec::new();
  let re = Regex::new(
    r#"(?P<num>[^。）]+)。(以下)?((?P<note>[^」）]+)において)?、?「(?P<name>[^」]*(法|令))」という。"#,
  )
  .unwrap();
  for caps in re.captures_iter(text) {
    let num_opt = caps.name("num").map(|n| n.as_str().to_string());
    let name_opt = caps.name("name").map(|n| n.as_str().to_string());
    let note_match = caps.name("note");
    let note = note_match.map(|n| n.as_str().to_string());
    match (num_opt, name_opt) {
      (Some(num), Some(name)) => lst.push(Ryakusyou { num, name, note }),
      _ => {}
    }
  }
  lst
}
