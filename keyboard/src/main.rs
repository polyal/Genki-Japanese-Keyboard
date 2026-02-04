use std::io;
use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Kana {
    key: String,
    value: String,
    #[serde(default)]
    next: Vec<Kana>,
}

fn iterate_kana(head: &Kana) {
    println!("key: {}", head.key);
    println!("value: {}", head.value);
    for child in &head.next {
        iterate_kana(child);
    }
}

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    println!("read then wrote: {buffer}");

    // json parsing
    let json = fs::read_to_string("kana/hiragana.json")
      .expect("couldnt read kana/hiragana.json");

    let roots = serde_json::from_str::<Kana>(&json).unwrap();
    
    // dbg!(&result);
    iterate_kana(&roots);
}
