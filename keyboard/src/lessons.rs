use std::fs;
use serde::Deserialize;
use std::io;
use rand::Rng;
use std::collections::HashSet;

use crate::RomanjiToKanaConverter;


struct Book {
  pub lessons: Vec<Lesson>,
}

impl Book {
  fn new() -> Self {
    // read genki lesson vocab
    let json = fs::read_to_string("resources/lessons.json")
      .expect("couldnt read resources/lessons.json");
    let lessons_wrapper: LessonsWrapper = serde_json::from_str::<LessonsWrapper>(&json).unwrap();
    
    Book {
      lessons: lessons_wrapper.lessons,
    }
  }
}

#[derive(Debug, Deserialize)]
struct LessonsWrapper {
  #[serde(default)]
  lessons: Vec<Lesson>,
}

#[derive(Debug, Deserialize)]
pub struct Lesson {
  index: usize,
  name_en: String,
  name_jp: String,
  #[serde(default)]
  vocab: Vec<Vocab>,
}

#[derive(Debug, Deserialize)]
pub struct Vocab {
  name: String,
  #[serde(default)]
  phrases: Vec<Phrase>,
}

#[derive(Debug, Deserialize)]
struct Phrase {
  en: String,
  jp: String,
  kanji: Option<String>,
}

pub struct Reviewer {
  book: Book,
}

impl Reviewer {
  pub fn new() -> Self {
    Reviewer {
      book: Book::new(),
    }
  }

  pub fn print_lessons(&self) {
    for lesson in &self.book.lessons { 
      println!("  [{}] {} - {}", lesson.index, lesson.name_en, lesson.name_jp);
    }
  }

  pub fn print_sections(&self, lesson: &Lesson) {
    let mut index: usize = 0;
    for vocab in &lesson.vocab { 
      println!("  [{index}] {}", vocab.name);
      index += 1;
    }
  }

  pub fn get_lesson(&self, index: usize) -> Option<&Lesson> {
    if index >= self.book.lessons.len() {
      return None;
    }
    return Some(&self.book.lessons[index]);
  }

  pub fn get_section<'a>(&self, lesson: &'a Lesson, index: usize) -> Option<&'a Vocab> {
    if index >= lesson.vocab.len() {
      return None;
    }
    return Some(&lesson.vocab[index]);
  }

  pub fn review_lesson(&self, lesson: &Lesson) {
    loop {
      let section_idx = rand::thread_rng().gen_range(0..lesson.vocab.len());
      let section = &lesson.vocab[section_idx];
      let phrase_idx = rand::thread_rng().gen_range(0..section.phrases.len());
      let phrase = &section.phrases[phrase_idx];
      if !self.review_phrase(&phrase) {
        return;
      }
    }
  }

  pub fn review_section(&self, section: &Vocab)
  {
    let mut asked: HashSet<usize> = HashSet::new();
    loop {
      if asked.len() == section.phrases.len() {
        asked.clear();
      }
      let mut phrase_idx = rand::thread_rng().gen_range(0..section.phrases.len());
      while !asked.insert(phrase_idx) {
        phrase_idx = rand::thread_rng().gen_range(0..section.phrases.len());
      }

      print!("\n  [{}/{}] ", phrase_idx, section.phrases.len(), );
      let phrase = &section.phrases[phrase_idx];
      if !self.review_phrase(&phrase) {
        break;
      }
    }
  }

  fn review_phrase(&self, phrase: &Phrase) -> bool {
    let mut buffer = String::new();
    let translate_direction = rand::thread_rng().gen_range(0..=1);
    if translate_direction == 0 {
      if let Some(kanji) = &phrase.kanji {
        println!("translate '{}' - '{}' to english", phrase.jp, kanji);
      }
      else {
        println!("translate '{}' to english", phrase.jp);
      }

      io::stdin().read_line(&mut buffer).expect("failed to read line");
      buffer.pop(); // remove '\n'

      if buffer == "exit" {
        return false;
      }

      println!("  your    answer: '{}'", buffer);        
      println!("  correct answer: '{}'", phrase.en);        
    }
    else {
      println!("translate '{}' to japanese", phrase.en);
      io::stdin().read_line(&mut buffer).expect("failed to read line");
      buffer.pop(); // remove '\n'

      if buffer == "exit" {
        return false;
      }

      let mut converter = RomanjiToKanaConverter::new();
      let kana = converter.convert(&buffer);

      println!("  your    answer: '{}'", kana);
      if let Some(kanji) = &phrase.kanji {
        println!("  correct answer: '{}' - '{}'", phrase.jp, kanji);
      }
      else {
        println!("  correct answer: '{}'", phrase.jp);
      } 
    }
    return true;
  }
}
