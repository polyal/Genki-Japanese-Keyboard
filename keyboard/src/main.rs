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

struct RomanjiToHiraganaConverter {
  head: Head,
  phrase: String,
}

impl RomanjiToHiraganaConverter {
  fn new(json: String) -> Self {
    RomanjiToHiraganaConverter {
      head: serde_json::from_str::<Head>(&json).unwrap(),
      phrase: String::new(),
    }
  }

  fn iterate_head<F>(head: &Head, cb: &mut F)
  where 
    F: FnMut(&Kana) -> bool, 
    {
      for root in &head.roots {
          Self::iterate_kana(root, cb);
      }
  }

  fn iterate_kana<F>(node: &Kana, cb: &mut F)
  where 
    F: FnMut(&Kana) -> bool, 
    {
      cb(&node);
      for child in &node.next {
          Self::iterate_kana(child, cb);
      }
  }

  fn convert(&mut self, romanji: &String) -> String {
    self.phrase = romanji.clone();
    let mut hiragana = String::new();

    let mut hiragana_creator = |node: &Kana| -> bool {
      let first = self.phrase.chars().next();
      if let Some(first) = &first {
        if node.key == *first {
          self.phrase.drain(..1);
          if *first == node.key {
            if let Some(value) = &node.value {
              hiragana.push_str(value);
              return true;
            }
          }
        }
      }
      return false;
    };

    Self::iterate_head(&self.head, &mut hiragana_creator);
    
    return hiragana;
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
