use std::fs;
use serde::Deserialize;


pub struct Book {
  pub lessons: Vec<Lesson>,
}

impl Book {
  pub fn new() -> Self {
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
  pub index: usize,
  pub name_en: String,
  pub name_jp: String,
  #[serde(default)]
  pub vocab: Vec<Vocab>,
}

#[derive(Debug, Deserialize)]
pub struct Vocab {
  pub name: String,
  #[serde(default)]
  pub phrases: Vec<Phrase>,
}

#[derive(Debug, Deserialize)]
pub struct Phrase {
  pub en: String,
  pub jp: String,
  pub kanji: Option<String>,
}
