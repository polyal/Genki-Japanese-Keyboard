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

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    println!("read then wrote: {buffer}");

    // json parsing
    let file = fs::File::open("kana/hiragana.json")
      .expect("file should open read only");
    let json = fs::read_to_string("kana/hiragana.json")
      .expect("couldnt read kana/hiragana.json");

    let result = serde_json::from_str::<Kana>(&json).unwrap();
    
    dbg!(&result);

    println!("{json}");
}
