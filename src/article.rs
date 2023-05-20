use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Article {
  /// 条
  pub article: Vec<usize>,
  /// 項
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub paragraph: Vec<usize>,
  /// 号
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub item: Vec<usize>,
  /// イロハなどの節（深さも記録する）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sub_item: Option<(usize, Vec<usize>)>,
  /// 附則の場合につける
  #[serde(skip_serializing_if = "Option::is_none")]
  pub suppl_provision_title: Option<String>,
}

impl Article {
  pub fn new() -> Self {
    Article {
      article: Vec::new(),
      paragraph: Vec::new(),
      item: Vec::new(),
      sub_item: None,
      suppl_provision_title: None,
    }
  }
  pub fn update_article(&mut self, num: &str) {
    let article_num = num
      .split('_')
      .map(|s| s.parse::<usize>().unwrap())
      .collect::<Vec<usize>>();
    *self = Article {
      article: article_num,
      paragraph: Vec::new(),
      item: Vec::new(),
      sub_item: None,
      ..self.clone()
    }
  }
  pub fn update_paragraph(&mut self, num: &str) {
    let paragraph_num = num
      .split('_')
      .map(|s| s.parse::<usize>().unwrap())
      .collect::<Vec<usize>>();
    *self = Article {
      paragraph: paragraph_num,
      item: Vec::new(),
      sub_item: None,
      ..self.clone()
    }
  }
  pub fn update_item(&mut self, num: &str) {
    let item_num = num
      .split('_')
      .map(|s| s.parse::<usize>().unwrap())
      .collect::<Vec<usize>>();
    *self = Article {
      item: item_num,
      sub_item: None,
      ..self.clone()
    }
  }
  pub fn update_sub_item(&mut self, n: usize, num: &str) {
    let sub_item_num = num
      .split('_')
      .map(|s| s.parse::<usize>().unwrap())
      .collect::<Vec<usize>>();
    *self = Article {
      sub_item: Some((n, sub_item_num)),
      ..self.clone()
    }
  }
  pub fn update_suppl_provision(&mut self, title: &str) {
    *self = Article {
      suppl_provision_title: Some(title.to_string()),
      ..Article::new()
    }
  }
}
