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
  hiragana: String
}

impl Phrase {
  fn new(phrase: &String) -> Self {
    Phrase {
      romanji: phrase.clone(),
      hiragana: String::new(),
    }
  }
}

struct RomanjiToHiraganaConverter {
  head: Head,
}

impl RomanjiToHiraganaConverter {
  fn new(json: String) -> Self {
    RomanjiToHiraganaConverter {
      head: serde_json::from_str::<Head>(&json).unwrap(),
    }
  }

  fn iterate_head(&self, phrase: &mut Phrase)
    {
      for root in &self.head.roots {
        if self.iterate_kana(root, phrase) {
          break;
        }
      }
    }

  fn iterate_kana(&self, node: &Kana, phrase: &mut Phrase) -> bool
    {
      let mut matched = false;
      if self.match_char(&node, phrase){
        for child in &node.next {
          if self.iterate_kana(child, phrase) {
            matched = true;
            break;
          }
        }
      }
      return matched;
    }

  fn match_char(&self, node: &Kana, phrase: &mut Phrase) -> bool {
    let first = phrase.romanji.chars().next();
    if let Some(first) = &first {
      if node.key == *first {
        phrase.romanji.drain(..1);
        if let Some(value) = &node.value {
          phrase.hiragana.push_str(value);
        }
        return true;
      }
    }
    return false;
  }

  fn convert(&mut self, romanji: &String) -> String {
    let mut phrase = Phrase::new(&romanji);

    while phrase.romanji.chars().count() > 1 {
      self.iterate_head(&mut phrase);
    }
    
    return phrase.hiragana.clone();
  }
}


fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    // read hiragana rules json
    let json = fs::read_to_string("kana/hiragana.json")
      .expect("couldnt read kana/hiragana.json");

    let mut converter = RomanjiToHiraganaConverter::new(json);
    let hiragana = converter.convert(&buffer);
    println!("converted {buffer} -> {hiragana}");
}
