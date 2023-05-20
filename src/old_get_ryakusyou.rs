use crate::range;
use regex::Regex;
use serde::{Deserialize, Serialize};

const RYAKUSYOU_RE: &str =
  r#"(?P<law_name>[^。）]+)。(以下)?((?P<note>[^」）]+)において)?「(?P<name>[^」]*(法|令))」という。"#;
const NOTE_RE_SUB : &str = "((?P<this_or_next>この|本|次)(?P<joukou_type>記載要領|条|項|号|節))|(?P<is_husoku>附則)?(?P<link>[次の第条項号節一-九十百千ア-ン]+)";

/// 以下[^」）]*号[^の及、まか並][^」）]*において「[^」]*法」という。


#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum JoukouType {
  /// 記載要領
  KisaiYouryou,
  /// 条
  Article,
  /// 項
  Paragraph,
  /// 号
  Item,
  /// 節
  SubItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum RyakusyouNoteType {
  /// この条や本項などの場合
  This(JoukouType),
  /// 次条や次項などの場合
  Next(JoukouType),
  /// 附則
  HusokuLink(String),
  /// 通常の条項
  Link(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum RyakusyouNote {
  Single {
    note: RyakusyouNoteType,
  },
  Range {
    start: RyakusyouNoteType,
    end: RyakusyouNoteType,
  },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Ryakusyou {
  name: String,
  note_lst: Vec<RyakusyouNote>,
  range: range::Range,
  law_article: Option<String>,
}

fn string_to_ryakusyou_note_type(text: &str) -> RyakusyouNoteType {
  let re = Regex::new(NOTE_RE_SUB).unwrap();
  let caps = re.captures(text).unwrap();
  if let Some(s) = caps.name("this_or_next").map(|m| m.as_str()) {
    let joukou_type = match caps.name("joukou_type").unwrap().as_str() {
      "記載要領" => JoukouType::KisaiYouryou,
      "条" => JoukouType::Article,
      "項" => JoukouType::Paragraph,
      "号" => JoukouType::Item,
      "節" => JoukouType::SubItem,
      _ => unreachable!(),
    };
    match s {
      "この" | "本" => RyakusyouNoteType::This(joukou_type),
      "次" => RyakusyouNoteType::Next(joukou_type),
      _ => unreachable!(),
    }
  } else {
    let link_str = caps.name("link").unwrap().as_str().to_string();
    if caps.name("is_husoku").is_some() {
      RyakusyouNoteType::HusokuLink(link_str)
    } else {
      RyakusyouNoteType::Link(link_str)
    }
  }
}

pub fn get_law_ryakusyou(law_article: Option<String>, text: &str) -> Vec<Ryakusyou> {
  let ryakusyou_re = Regex::new(RYAKUSYOU_RE).unwrap();
  let note_re = Regex::new(&format!(
    "(、|及び|並びに)?(?P<start>{NOTE_RE_SUB})(から(?P<end>{NOTE_RE_SUB})まで)?"
  ))
  .unwrap();
  let mut lst = Vec::new();
  for caps in ryakusyou_re.captures_iter(text) {
    let name_match = caps.name("name").unwrap();
    let range = range::match_to_range(&name_match);
    let name = name_match.as_str().to_string();
    if let Some(note) = caps.name("note") {
      let s = note.as_str();
      let mut note_lst = Vec::new();
      for note_caps in note_re.captures_iter(s) {
        let start_match = note_caps.name("start");
        let end_match = note_caps.name("end");
        match (start_match, end_match) {
          (Some(s), Some(e)) => note_lst.push(RyakusyouNote::Range {
            start: string_to_ryakusyou_note_type(s.as_str()),
            end: string_to_ryakusyou_note_type(e.as_str()),
          }),
          (Some(s), None) => note_lst.push(RyakusyouNote::Single {
            note: string_to_ryakusyou_note_type(s.as_str()),
          }),
          _ => unreachable!(),
        }
      }
      lst.push(Ryakusyou {
        name,
        note_lst,
        range,
        law_article: law_article.clone(),
      })
    } else {
      lst.push(Ryakusyou {
        name,
        note_lst: Vec::new(),
        range,
        law_article: law_article.clone(),
      })
    }
  }
  lst
}
