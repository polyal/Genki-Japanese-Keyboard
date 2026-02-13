use std::fs;
use serde::Deserialize;
use std::io;
use rand::Rng;
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::RomanjiToKanaConverter;

static mut CONVERTER: LazyLock<RomanjiToKanaConverter> = LazyLock::new(|| RomanjiToKanaConverter::new());


struct Book {
  lessons: Vec<Lesson>,
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
struct Lesson {
  index: usize,
  name_en: String,
  name_jp: String,
  #[serde(default)]
  sections: Vec<Section>,
}

#[derive(Debug, Deserialize)]
struct Section {
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

  pub fn start(&self) {
    let mut buffer = String::new();
    while buffer != ":q" {
      buffer.clear();
      // pick lesson
      let mut lesson_idx = usize::MAX;
      loop {
        if let Some(lesson) = &self.get_lesson(lesson_idx) {
          let mut section_idx: usize;
          loop {
            // test section
            println!("\nPick a section: ");
            self.print_sections(&lesson);   

            buffer.clear();
            io::stdin().read_line(&mut buffer).expect("failed to read line");
            buffer.pop(); // remove '\n'
            match buffer.parse::<usize>() {
              Ok(n) => section_idx = n,
              Err(_e) => {
                if buffer == ":b" {
                  break;
                }
                else {
                  continue;
                }
              },
            }
            if let Some(section) = &self.get_section(lesson, section_idx) {
              self.review_section(section);
            }
            else {
              self.review_lesson(lesson);
            }
          }
        }

        // lesson selection
        println!("\nPick a lesson: ");
        self.print_lessons();

        buffer.clear();
        io::stdin().read_line(&mut buffer).expect("failed to read line");
        buffer.pop(); // remove '\n'
        match buffer.parse::<usize>() {
          Ok(n) => lesson_idx = n,
          Err(_e) => break,
        }
      }
    }
  }

  fn print_lessons(&self) {
    for lesson in &self.book.lessons { 
      println!("  [{}] {} - {}", lesson.index, lesson.name_en, lesson.name_jp);
    }
  }

  fn print_sections(&self, lesson: &Lesson) {
    let mut index: usize = 0;
    for section in &lesson.sections { 
      println!("  [{index}] {}", section.name);
      index += 1;
    }
  }

  fn get_lesson(&self, index: usize) -> Option<&Lesson> {
    if index >= self.book.lessons.len() {
      return None;
    }
    return Some(&self.book.lessons[index]);
  }

  fn get_section<'a>(&self, lesson: &'a Lesson, index: usize) -> Option<&'a Section> {
    if index >= lesson.sections.len() {
      return None;
    }
    return Some(&lesson.sections[index]);
  }

  fn review_lesson(&self, lesson: &Lesson) {
    loop {
      let section_idx = rand::thread_rng().gen_range(0..lesson.sections.len());
      let section = &lesson.sections[section_idx];
      let phrase_idx = rand::thread_rng().gen_range(0..section.phrases.len());
      let phrase = &section.phrases[phrase_idx];
      if !self.review_phrase(&phrase) {
        return;
      }
    }
  }

  fn review_section(&self, section: &Section)
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

      if buffer == ":b" {
        return false;
      }

      println!("  your    answer: '{}'", buffer);        
      println!("  correct answer: '{}'", phrase.en);        
    }
    else {
      println!("translate '{}' to japanese", phrase.en);
      io::stdin().read_line(&mut buffer).expect("failed to read line");
      buffer.pop(); // remove '\n'

      if buffer == ":b" {
        return false;
      }

      let converter_ptr = std::ptr::addr_of_mut!(CONVERTER);
      unsafe {
        let kana = (*converter_ptr).convert(&buffer);
        println!("  your    answer: '{}'", kana);
      }
      
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
