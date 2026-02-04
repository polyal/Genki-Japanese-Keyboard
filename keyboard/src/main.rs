use std::io;

// const HIRAGANA_A: [(char, char); 1] = [('a', 'あ')];
// const HIRAGANA_I: [(char, char); 1] = [('i', 'い')];
// const HIRAGANA_U: [(char, char); 1] = [('u', 'う')];
// const HIRAGANA_E: [(char, char); 1] = [('e', 'え')];
// const HIRAGANA_O: [(char, char); 1] = [('o', 'お')];

const HIRAGANA_CHART : &str = r#"
{
  "a": "あ",
  "i": "い",
  "u": "う",
  "e": "え",
  "o": "お",
  "k": [
    {
      "a": "か"
    },
    {
      "i": "き"
    },
    {
      "u": "く"
    },
    {
      "e": "け"
    },
    {
      "o": "こ"
    },
    {
      "y": [
        {
          "a": "ゃ"
        },
        {
          "u": "ゅ"
        },
        {
          "o": "ょ"
        }
      ]
    }
  ]
}
"#;

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    println!("read then wrote: {buffer}");

    let json: serde_json::Value =
        serde_json::from_str(HIRAGANA_CHART).expect("JSON was not well-formatted");

    println!("{json}");
}
