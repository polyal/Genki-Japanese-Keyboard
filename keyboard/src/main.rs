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

fn iterate_kana<F>(head: &Kana, f: &mut F)
  where F: FnMut(Option<char>, Option<char>),
  {
    f(head.value, head.ext);
    for child in &head.next {
        iterate_kana(child, f);
    }
}

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    println!("read then wrote: {buffer}");

    // json parsing
    let json = fs::read_to_string("kana/hiragana.json")
      .expect("couldnt read kana/hiragana.json");

    let mut single_hiragana = String::new();

    let mut string_pusher = move |value: Option<char>, ext: Option<char>| {
      if let Some(value) = value {
        single_hiragana.push(value);
        if let Some(ext) = ext {
          single_hiragana.push(ext);
        }
      }
      println!("{single_hiragana}");
    };

    let head = serde_json::from_str::<Kana>(&json).unwrap();
    
    // dbg!(&result);
    iterate_kana(&head, &mut string_pusher);
}
