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

fn iterate_head<F>(head: &Head, cb: &mut F)
where 
  F: FnMut(&Kana, bool), 
  {
    for root in &head.roots {
        iterate_kana(root, cb);
    }
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
    let mut chars_pushed: Vec<usize> = Vec::new();

    let mut hiragana_creator = |node: &Kana, pre: bool| {
      if pre {
        let mut pushed_size: usize = 0;
        if let Some(value) = &node.value {
          single_hiragana.push_str(&value);
          pushed_size = value.chars().count();
          println!("{single_hiragana}");
        }
        chars_pushed.push(pushed_size);
      }
      else {
        let pushed_size = chars_pushed.pop();
        for _i in 1..=pushed_size.unwrap() {
          single_hiragana.pop();
        }
      }
    };

    let head = serde_json::from_str::<Head>(&json).unwrap();
    
    // dbg!(&result);
    iterate_head(&head, &mut hiragana_creator);
}
