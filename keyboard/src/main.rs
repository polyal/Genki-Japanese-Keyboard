use std::io;
use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Kana {
    key: char,
    value: Option<char>,
    ext: Option<char>,
    #[serde(default)]
    next: Vec<Kana>,
}

fn iterate_kana<F>(head: &Kana, cb: &mut F)
where 
  F: FnMut(&Kana, bool), 
  {
    cb(&head, true);
    for child in &head.next {
        iterate_kana(child, cb);
    }
    cb(&head, false);
}

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    println!("read then wrote: {buffer}");

    // json parsing
    let json = fs::read_to_string("kana/hiragana.json")
      .expect("couldnt read kana/hiragana.json");

    let mut single_hiragana = String::new();
    let mut chars_pushed: Vec<u8> = Vec::new();

    let mut hiragana_creator = |node: &Kana, pre: bool| {
      if pre {
        let mut num_pushed: u8 = 0;
        if let Some(value) = node.value {
          single_hiragana.push(value);
          num_pushed += 1;
          if let Some(ext) = node.ext {
            single_hiragana.push(ext);
            num_pushed += 1;
          }
          println!("{single_hiragana}");
        }
        chars_pushed.push(num_pushed);
      }
      else {
        let num_pushed = chars_pushed.pop();
        for _i in 1..=num_pushed.unwrap() {
          single_hiragana.pop();
        }
      }
    };

    let head = serde_json::from_str::<Kana>(&json).unwrap();
    
    // dbg!(&result);
    iterate_kana(&head, &mut hiragana_creator);
}
