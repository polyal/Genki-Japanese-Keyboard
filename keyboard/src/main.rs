mod converter;
mod lessons;

use converter::RomanjiToKanaConverter;
use lessons::Book;
use std::io;

fn main() {
  let mut buffer = String::new();
  io::stdin().read_line(&mut buffer).expect("failed to read line");
  buffer.pop(); // remove '\n'

  let mut converter = RomanjiToKanaConverter::new();
  let kana = converter.convert(&buffer);
  println!("converted '{buffer}' -> '{kana}'");

  let book = Book::new();
  for lesson in &book.lessons.lessons {
    println!("lesson_{}: {}/{}", lesson.index, lesson.name_en, lesson.name_jp);
    for vocab in &lesson.vocab {
      println!("  section: {}", vocab.name);
      for phrase in &vocab.phrases {
        println!("    '{}' - '{}'", phrase.en, phrase.jp);
      }
    }
  }
}
