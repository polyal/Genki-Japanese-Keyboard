use std::fs;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct Head {
  #[serde(default)]
  roots: Vec<Kanji>,
}

#[derive(Debug, Deserialize)]
struct Kanji {
  key: char,
  value: Option<Vec<char>>,
  #[serde(default)]
  next: Vec<Kanji>,
}

struct Phrase<'a> {
  hiragana: &'a String,
  kanji: Vec<char>,
  offset: usize,
}

impl <'a> Phrase<'a> {
  fn new(phrase: &'a String) -> Self {
    Phrase {
      hiragana: phrase,
      kanji: Vec::<char>::new(),
      offset: 0,
    }
  }

  fn compare(&mut self, node: &Kanji) -> bool {
    let first = self.hiragana.chars().nth(self.offset);
    if let Some(first) = &first {
      if node.key == *first {
        self.offset += 1;
        if self.done() && let Some(value) = &node.value {
          self.kanji = value.clone();
        }
        else if self.done() {
          return false;
        }
        return true;
      }
    }
    return false;
  }

  fn done(&self) -> bool {
    return self.offset == self.hiragana.chars().count();
  }

  fn reset(&mut self) {
    self.offset = 0;
  }

  fn get_kanji(self) -> Vec<char> {
    return self.kanji;
  }
}

pub struct HiragaToKanjiConverter {
  head: Head,
}

impl HiragaToKanjiConverter {
  pub fn new() -> Self {
    // read kanji rules json
    let json = fs::read_to_string("resources/kanji.json")
      .expect("couldnt read resources/kanji.json");
    
    HiragaToKanjiConverter {
      head: serde_json::from_str::<Head>(&json).unwrap(),
    }
  }

  fn convert_phrase(&self, phrase: &mut Phrase) {
    for root in &self.head.roots {
      if self.iterate_kanji(root, phrase) {
        break;
      }
      else {
        // not matched on current root
        // reset buffer and try next root
        phrase.reset();
      }
    }
  }

  fn iterate_kanji(&self, node: &Kanji, phrase: &mut Phrase) -> bool {
    let mut matched = phrase.compare(&node);
    if matched && !phrase.done() {
      for child in &node.next {
        matched = self.iterate_kanji(child, phrase);
        if matched {
          break;
        }
      }
    }
    return matched;
  }

  pub fn convert(&self, hiragana: &String) -> Vec<char> {
    let mut phrase = Phrase::new(&hiragana);
    self.convert_phrase(&mut phrase);
    return phrase.get_kanji();
  }
}
