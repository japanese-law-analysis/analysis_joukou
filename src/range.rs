use regex::Match;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Range {
  start: usize,
  end: usize,
}

pub fn match_to_range(m: &Match) -> Range {
  Range {
    start: m.start(),
    end: m.end(),
  }
}
