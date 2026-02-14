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

struct Phrase<'a> {
  romanji: &'a String,
  kana: String,
  offset: usize,
  len: usize,
}

impl <'a> Phrase<'a> {
  fn new(phrase: &'a String) -> Self {
    Phrase {
      romanji: phrase,
      kana: String::new(),
      offset: 0,
      len: 0
    }
  }

  fn compare(&mut self, node: &Kana) -> bool {
    let first = self.romanji.chars().nth(self.offset + self.len);
    if let Some(first) = &first {
      if node.key == *first {
        self.len += 1;
        if let Some(value) = &node.value {
          self.kana.push_str(value);
        }
        return true;
      }
    }
    return false;
  }

  fn done(&self) -> bool {
    return self.offset >= self.romanji.len();
  }

  fn next(&mut self) {
    self.offset += self.len;
    self.len = 0;
  }

  fn skip(&mut self) {
    // cant match romanji
    // push unmatched char onto result
    let first = self.romanji.chars().nth(self.offset);
    if let Some(first) = &first {
      self.kana.push(*first);
      // continue matching with next char
      self.offset += 1;
      self.len = 0;
    }
  }

  fn reset(&mut self) {
    self.len = 0;
  }

  fn get_kana(self) -> String {
    return self.kana;
  }
}

pub struct RomanjiToKanaConverter {
  head: Head,
}

impl RomanjiToKanaConverter {
  pub fn new() -> Self {
  	// read hiragana/katakana rules json
	  let json = fs::read_to_string("resources/kana.json")
	    .expect("couldnt read resources/kana.json");
    
    RomanjiToKanaConverter {
      head: serde_json::from_str::<Head>(&json).unwrap(),
    }
  }

  fn convert_phrase(&self, phrase: &mut Phrase) -> bool {
    for root in &self.head.roots {
      if self.iterate_kana(root, phrase) {
        return true;
      }
      else {
        // not matched on current root
        // reset buffer and try next root
        phrase.reset();
      }
    }
    return false;
  }

  fn iterate_kana(&self, node: &Kana, phrase: &mut Phrase) -> bool {
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

  pub fn convert(&self, romanji: &String) -> String {
    let mut phrase = Phrase::new(&romanji);
    while !phrase.done() {
      if self.convert_phrase(&mut phrase) {
        phrase.next();
      }
      else {
        phrase.skip();
      }
    }
    return phrase.get_kana();
  }
}


#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_converter() {
    let converter = RomanjiToKanaConverter::new();
    let test_cases = vec![
      ("a", "あ"),
      ("i", "い"),
      ("u", "う"),
      ("e", "え"),
      ("o", "お"),
      ("ka", "か"),
      ("ki", "き"),
      ("ku", "く"),
      ("ke", "け"),
      ("ko", "こ"),
      ("kya", "きゃ"),
      ("kyu", "きゅ"),
      ("kyo", "きょ"),
      ("kka", "っか"),
      ("kki", "っき"),
      ("kku", "っく"),
      ("kke", "っけ"),
      ("kko", "っこ"),
      ("kkya", "っきゃ"),
      ("kkyu", "っきゅ"),
      ("kkyo", "っきょ"),
      ("sa", "さ"),
      ("shi", "し"),
      ("sha", "しゃ"),
      ("shu", "しゅ"),
      ("sho", "しょ"),
      ("su", "す"),
      ("se", "せ"),
      ("so", "そ"),
      ("ssa", "っさ"),
      ("sshi", "っし"),
      ("ssha", "っしゃ"),
      ("sshu", "っしゅ"),
      ("ssho", "っしょ"),
      ("ssu", "っす"),
      ("sse", "っせ"),
      ("sso", "っそ"),
      ("ta", "た"),
      ("tsu", "つ"),
      ("te", "て"),
      ("to", "と"),
      ("tta", "った"),
      ("ttsu", "っつ"),
      ("tte", "って"),
      ("tto", "っと"),
      ("chi", "ち"),
      ("cha", "ちゃ"),
      ("chu", "ちゅ"),
      ("cho", "ちょ"),
      ("cchi", "っち"),
      ("ccha", "っちゃ"),
      ("cchu", "っちゅ"),
      ("ccho", "っちょ"),
      ("na", "な"),
      ("ni", "に"),
      ("nu", "ぬ"),
      ("ne", "ね"),
      ("no", "の"),
      ("nya", "にゃ"),
      ("nyu", "にゅ"),
      ("nyo", "にょ"),
      ("ha", "は"),
      ("hi", "ひ"),
      ("he", "へ"),
      ("ho", "ほ"),
      ("hya", "ひゃ"),
      ("hyu", "ひゅ"),
      ("hyo", "ひょ"),
      ("fu", "ふ"),
      ("ma", "ま"),
      ("mi", "み"),
      ("mu", "む"),
      ("me", "め"),
      ("mo", "も"),
      ("mya", "みゃ"),
      ("myu", "みゅ"),
      ("myo", "みょ"),
      ("ya", "や"),
      ("yu", "ゆ"),
      ("yo", "よ"),
      ("ra", "ら"),
      ("ri", "り"),
      ("ru", "る"),
      ("re", "れ"),
      ("ro", "ろ"),
      ("rya", "りゃ"),
      ("ryu", "りゅ"),
      ("ryo", "りょ"),
      ("wa", "わ"),
      ("wo", "を"),
      ("n", "ん"),
      ("ga", "が"),
      ("gi", "ぎ"),
      ("gu", "ぐ"),
      ("ge", "げ"),
      ("go", "ご"),
      ("gya", "ぎゃ"),
      ("gyu", "ぎゅ"),
      ("gyo", "ぎょ"),
      ("za", "ざ"),
      ("zu", "ず"),
      ("ze", "ぜ"),
      ("zo", "ぞ"),
      ("ji", "じ"),
      ("ja", "じゃ"),
      ("ju", "じゅ"),
      ("jo", "じょ"),
      ("da", "だ"),
      ("de", "で"),
      ("do", "ど"),
      ("ba", "ば"),
      ("bi", "び"),
      ("bu", "ぶ"),
      ("be", "べ"),
      ("bo", "ぼ"),
      ("bya", "びゃ"),
      ("byu", "びゅ"),
      ("byo", "びょ"),
      ("pa", "ぱ"),
      ("pi", "ぴ"),
      ("pu", "ぷ"),
      ("pe", "ぺ"),
      ("po", "ぽ"),
      ("pya", "ぴゃ"),
      ("pyu", "ぴゅ"),
      ("pyo", "ぴょ"),
      ("ppa", "っぱ"),
      ("ppi", "っぴ"),
      ("ppu", "っぷ"),
      ("ppe", "っぺ"),
      ("ppo", "っぽ"),
      ("ppya", "っぴゃ"),
      ("ppyu", "っぴゅ"),
      ("ppyo", "っぴょ"),
      ("A", "ア"),
      ("I", "イ"),
      ("U", "ウ"),
      ("E", "エ"),
      ("O", "オ"),
      ("KA", "カ"),
      ("KI", "キ"),
      ("KU", "ク"),
      ("KE", "ケ"),
      ("KO", "コ"),
      ("KYA", "キャ"),
      ("KYU", "キュ"),
      ("KYO", "キョ"),
      ("KKA", "ッカ"),
      ("KKI", "ッキ"),
      ("KKU", "ック"),
      ("KKE", "ッケ"),
      ("KKO", "ッコ"),
      ("KKYA", "ッキャ"),
      ("KKYU", "ッキュ"),
      ("KKYO", "ッキョ"),
      ("KWA", "クァ"),
      ("KWI", "クィ"),
      ("KWE", "クェ"),
      ("KWO", "クォ"),
      ("SA", "サ"),
      ("SHI", "シ"),
      ("SHA", "シャ"),
      ("SHU", "シュ"),
      ("SHE", "シェ"),
      ("SHO", "ショ"),
      ("SU", "ス"),
      ("SE", "セ"),
      ("SO", "ソ"),
      ("SSA", "ッさ"),
      ("SSHI", "ッシ"),
      ("SSHA", "ッシャ"),
      ("SSHU", "ッシュ"),
      ("SSHE", "ッシェ"),
      ("SSHO", "ッショ"),
      ("SSU", "ッス"),
      ("SSE", "ッセ"),
      ("SSO", "ッソ"),
      ("TA", "タ"),
      ("TSA", "ツァ"),
      ("TSI", "ツィ"),
      ("TSU", "ツ"),
      ("TSE", "ツェ"),
      ("TSO", "ツォ"),
      ("TI", "ティ"),
      ("TU", "トゥ"),
      ("TE", "テ"),
      ("TO", "ト"),
      ("TYU", "テュ"),
      ("TTA", "ッタ"),
      ("TTSA", "ッツァ"),
      ("TTSI", "ッツィ"),
      ("TTSU", "ッツ"),
      ("TTSE", "ッツェ"),
      ("TTSO", "ッツォ"),
      ("TTI", "ッティ"),
      ("TTU", "ットゥ"),
      ("TTE", "ッテ"),
      ("TTO", "ット"),
      ("TTYU", "ッテュ"),
      ("CHI", "チ"),
      ("CHA", "チャ"),
      ("CHU", "チュ"),
      ("CHE", "チェ"),
      ("CHO", "チョ"),
      ("CCHI", "ッチ"),
      ("CCHA", "ッチャ"),
      ("CCHU", "ッチュ"),
      ("CCHE", "ッチェ"),
      ("CCHO", "ッチョ"),
      ("NA", "ナ"),
      ("NI", "ニ"),
      ("NU", "ヌ"),
      ("NE", "ネ"),
      ("NO", "ノ"),
      ("NYA", "ニャ"),
      ("NYU", "ニュ"),
      ("NYO", "ニョ"),
      ("HA", "ハ"),
      ("HI", "ヒ"),
      ("HE", "ヘ"),
      ("HO", "ホ"),
      ("HYA", "ヒャ"),
      ("HYU", "ヒュ"),
      ("HYO", "ヒョ"),
      ("FA", "ファ"),
      ("FI", "フィ"),
      ("FU", "フ"),
      ("FE", "フェ"),
      ("FO", "フォ"),
      ("FYU", "フュ"),
      ("MA", "マ"),
      ("MI", "ミ"),
      ("MU", "ム"),
      ("ME", "メ"),
      ("MO", "モ"),
      ("MYA", "ミャ"),
      ("MYU", "ミュ"),
      ("MYO", "ミョ"),
      ("YA", "ヤ"),
      ("YU", "ユ"),
      ("YE", "イェ"),
      ("YO", "ヨ"),
      ("RA", "ラ"),
      ("RI", "リ"),
      ("RU", "ル"),
      ("RE", "レ"),
      ("RO", "ロ"),
      ("RYA", "リャ"),
      ("RYU", "リュ"),
      ("RYO", "リョ"),
      ("WA", "ワ"),
      ("WI", "ウィ"),
      ("WE", "ウェ"),
      ("WO", "ヲ"),
      ("N", "ン"),
      ("GA", "ガ"),
      ("GI", "ギ"),
      ("GU", "グ"),
      ("GE", "ゲ"),
      ("GO", "ゴ"),
      ("GYA", "ギャ"),
      ("GYU", "ギュ"),
      ("GYO", "ギョ"),
      ("GWA", "グァ"),
      ("ZA", "ザ"),
      ("ZU", "ズ"),
      ("ZE", "ゼ"),
      ("ZO", "ゾ"),
      ("JI", "ジ"),
      ("JA", "ジャ"),
      ("JU", "ジュ"),
      ("JE", "ジェ"),
      ("JO", "ジョ"),
      ("DA", "ダ"),
      ("DI", "ディ"),
      ("DU", "ドゥ"),
      ("DE", "デ"),
      ("DO", "ド"),
      ("DYU", "デュ"),
      ("BA", "バ"),
      ("BI", "ビ"),
      ("BU", "ブ"),
      ("BE", "ベ"),
      ("BO", "ボ"),
      ("BYA", "ビャ"),
      ("BYU", "ビュ"),
      ("BYO", "ビョ"),
      ("PA", "パ"),
      ("PI", "ピ"),
      ("PU", "プ"),
      ("PE", "ペ"),
      ("PO", "ポ"),
      ("PYA", "ピャ"),
      ("PYU", "ピュ"),
      ("PYO", "ピョ"),
      ("PPA", "ッパ"),
      ("PPI", "ッピ"),
      ("PPU", "ップ"),
      ("PPE", "ッペ"),
      ("PPO", "ッポ"),
      ("PPYA", "ッピャ"),
      ("PPYU", "ッピュ"),
      ("PPYO", "ッピョ"),
      ("VA", "ヴァ"),
      ("VI", "ヴィ"),
      ("VU", "ヴ"),
      ("VE", "ヴェ"),
      ("VO", "ヴォ"),
      ("VYU", "ヴゥ"),
      ("-", "ー"),
      (".", "。"),
    ];

    for (romaji, expected) in test_cases {
      let result = converter.convert(&romaji.to_string());
      assert_eq!(result, expected, "Failed for romaji: {}", romaji);
    }
  }
}
