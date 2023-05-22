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
    r#"（(?P<num>(明治|大正|昭和|平成|令和)(一|二|三|四|五|六|七|八|九|十|〇)+年[^（）、。あ-ん]+第(一|二|三|四|五|六|七|八|九|十|百|千|〇)+号)。?(以下)?((?P<note>[^」）]+)において)?、?「(?P<name>[^」]*(法|令))」という。"#,
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

#[tokio::test]
async fn check_get_law_abbreviation_1() {
  let a = Article {
    article: String::new(),
    paragraph: None,
    item: None,
    sub_item: None,
    suppl_provision_title: None,
  };
  let v = get_law_abbreviation("", &a, "附則第七条の二及び第七条の三の規定の適用がないものとした場合における地方交付税法等の一部を改正する法律（平成三十一年法律第五号）第三条の規定による改正前の地方特例交付金等の地方財政の特別措置に関する法律（平成十一年法律第十七号）第八条第一項及び地方税法等の一部を改正する等の法律（平成二十八年法律第十三号。以下イにおいて「平成二十八年地方税法等改正法」という。）第九条の規定による廃止前の地方法人特別税等に関する暫定措置法（平成二十年法律第二十五号）第三十九条の規定により読み替えられた平成二十八年地方税法等改正法附則第三十七条の規定による改正前の地方交付税法第十四条（以下この条において「読替え後の地方交付税法第十四条」という。）");
  assert_eq!(
    v,
    vec![Abbreviation {
      num: "平成二十八年法律第十三号".to_string(),
      name: "平成二十八年地方税法等改正法".to_string(),
      note: Some("イ".to_string()),
      article: a,
      law_num: "".to_string(),
    },]
  )
}
