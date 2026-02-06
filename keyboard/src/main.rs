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
      let mut matched = self.match_char(&node, phrase);
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

  fn match_char(&self, node: &Kana, phrase: &mut Phrase) -> bool {
    let first = phrase.buffer.chars().next();
    if let Some(first) = &first {
      if node.key == *first {
        phrase.buffer.drain(..1);
        if let Some(value) = &node.value {
          phrase.kana.push_str(value);
        }
        return true;
      }
    }
    return false;
  }

  fn convert(&mut self, romanji: &String) -> String {
    let mut phrase = Phrase::new(&romanji);

    while phrase.romanji.chars().count() > 1 {
      if self.iterate_head(&mut phrase) {
        // matched romanji
        // buffer holds remainng phrase to be converted
        phrase.romanji = phrase.buffer.clone();
      }
      else {
        // cant match romanji
        // push unmatched char onto result
        let first = phrase.buffer.chars().next();
        if let Some(first) = &first {
          phrase.kana.push(*first);
        }
        // continue match with next char
        phrase.romanji.drain(..1);
        phrase.buffer = phrase.romanji.clone();
      }
    }
    
    return phrase.kana.clone();
  }
}


fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    // read hiragana rules json
    let json = fs::read_to_string("kana/hiragana.json")
      .expect("couldnt read kana/hiragana.json");

    let mut converter = RomanjiToKanaConverter::new(json);
    let kana = converter.convert(&buffer);
    println!("converted {buffer} -> {kana}");
}
