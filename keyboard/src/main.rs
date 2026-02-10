mod converter;
mod lessons;

use converter::RomanjiToKanaConverter;
use lessons::Book;
use std::io;
use rand::Rng;

fn main() {
  let mut buffer = String::new();
  println!("!!Genki-Japanese-Keyboard!!");


  let book = Book::new();
  while buffer != "exit" {
    buffer.clear();
    // pick lesson
    let mut lesson_idx: usize = book.lessons.len();
    while lesson_idx >= book.lessons.len(){
      println!("\nPick a lesson: ");
      let mut idx: usize = 0;
      for lesson in &book.lessons { 
        println!("  [{idx}] {} - {}", lesson.name_en, lesson.name_jp);
        idx += 1;
      }
      io::stdin().read_line(&mut buffer).expect("failed to read line");
      buffer.pop(); // remove '\n'
      match buffer.parse::<usize>() {
        Ok(n) => lesson_idx = n,
        Err(_e) => break,
      }
      buffer.clear();
    }

    if lesson_idx >= book.lessons.len() {
      break;
    }
    
    // pick section
    let lesson = &book.lessons[lesson_idx];
    let mut section_idx: usize = lesson.vocab.len();
    while section_idx >= lesson.vocab.len(){
      println!("\nPick a section: ");
      let mut idx: usize = 0;
      for section in &lesson.vocab { 
        println!("  [{idx}] {}", section.name);
        idx += 1;
      }
      io::stdin().read_line(&mut buffer).expect("failed to read line");
      buffer.pop(); // remove '\n'
      match buffer.parse::<usize>() {
        Ok(n) => section_idx = n,
        Err(_e) => break,
      }
      buffer.clear();
    }

    if section_idx >= lesson.vocab.len() {
      break;
    }

    // test section
    let section = &lesson.vocab[section_idx];
    loop {
      let phrase_idx: usize = rand::thread_rng().gen_range(0..section.phrases.len());
      let phrase = &section.phrases[phrase_idx];

      let translate_direction: usize = rand::thread_rng().gen_range(0..=1);
      if translate_direction == 0 {
        println!("\n  [{}/{}]translate {} to english",phrase_idx, section.phrases.len(), phrase.jp);
        io::stdin().read_line(&mut buffer).expect("failed to read line");
        buffer.pop(); // remove '\n'

        if buffer == "exit" {
          break;
        }

        println!("  your    answer: {}", buffer);        
        println!("  correct answer: {}", phrase.en);        
      }
      else {
        println!("\n  [{}/{}]translate {} to japanese", phrase_idx, section.phrases.len(), phrase.en);
        io::stdin().read_line(&mut buffer).expect("failed to read line");
        buffer.pop(); // remove '\n'

        if buffer == "exit" {
          break;
        }

        let mut converter = RomanjiToKanaConverter::new();
        let kana = converter.convert(&buffer);

        println!("  your    answer: {}", kana);        
        println!("  correct answer: {}", phrase.jp);
      }
      buffer.clear();
    }
  }
}
