use std::io;
use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Head {
    #[serde(default)]
    roots: Vec<Kana>,
}

#[derive(Debug, Deserialize)]
struct Kana {
    key: char,
    value: Option<String>,
    #[serde(default)]
    next: Vec<Kana>,
}

struct Phrase {
  romanji: String,
  buffer: String,
  kana: String,
}

impl Phrase {
  fn new(phrase: &String) -> Self {
    Phrase {
      romanji: phrase.clone(),
      buffer: phrase.clone(),
      kana: String::new(),
    }
  }

  fn compare(&mut self, node: &Kana) -> bool {
    let first = self.buffer.chars().next();
    if let Some(first) = &first {
      if node.key == *first {
        self.buffer.drain(..1);
        if let Some(value) = &node.value {
          self.kana.push_str(value);
        }
        return true;
      }
    }
    return false;
  }

  fn done(&self) -> bool {
    // last char is new line so dont consume
    return !(self.romanji.chars().count() > 1);
  }

  fn next(&mut self) {
    // matched romanji
    // buffer holds remainng phrase to be converted
    self.romanji = self.buffer.clone();
  }

  fn skip(&mut self) {
    // cant match romanji
    // push unmatched char onto result
    let first = self.romanji.chars().next();
    if let Some(first) = &first {
      self.kana.push(*first);
      // continue match with next char
      self.romanji.drain(..1);
      self.buffer = self.romanji.clone();
    }
  }
}

struct RomanjiToKanaConverter {
  head: Head,
}

impl RomanjiToKanaConverter {
  fn new(json: String) -> Self {
    RomanjiToKanaConverter {
      head: serde_json::from_str::<Head>(&json).unwrap(),
    }
  }

  fn iterate_head(&self, phrase: &mut Phrase) -> bool
    {
      for root in &self.head.roots {
        if self.iterate_kana(root, phrase) {
          return true;
        }
        else {
          // not matched on current root
          // reset buffer and try next root
          phrase.buffer = phrase.romanji.clone();
        }
      }
      return false;
    }

  fn iterate_kana(&self, node: &Kana, phrase: &mut Phrase) -> bool
    {
      let mut matched = phrase.compare(&node);
      if matched {
        for child in &node.next {
          matched = self.iterate_kana(child, phrase);
          if matched {
            break;
          }
        }
      }
      return matched;
    }

  fn convert(&mut self, romanji: &String) -> String {
    let mut phrase = Phrase::new(&romanji);

    while !phrase.done() {
      if self.iterate_head(&mut phrase) {
        phrase.next();
      }
      else {
        phrase.skip();
      }
    }
    
    return phrase.kana.clone();
  }
}


fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    // read hiragana/katakana rules json
    let json = fs::read_to_string("kana/rules.json")
      .expect("couldnt read kana/rules.json");

    let mut converter = RomanjiToKanaConverter::new(json);
    let kana = converter.convert(&buffer);
    println!("converted {buffer} -> {kana}");
}
