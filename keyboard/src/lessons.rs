use serde::Deserialize;
use std::fs;

pub struct Book {
    lessons: Vec<Lesson>,
}

impl Book {
    pub fn new() -> Self {
        // read genki lesson vocab
        let json = fs::read_to_string("resources/lessons.json")
            .expect("couldnt read resources/lessons.json");
        let lessons_wrapper: LessonsWrapper =
            serde_json::from_str::<LessonsWrapper>(&json).unwrap();

        Book {
            lessons: lessons_wrapper.lessons,
        }
    }

    pub fn get_lessons(&self) -> &Vec<Lesson> {
        return &self.lessons;
    }

    pub fn get_lesson(lessons: &Vec<Lesson>, index: usize) -> Option<&Lesson> {
        if index >= lessons.len() {
            return None;
        }
        return Some(&lessons[index]);
    }

    pub fn get_sections(lesson: &Lesson) -> &Vec<Section> {
        return &lesson.sections;
    }

    pub fn get_section<'a>(lesson: &'a Lesson, index: usize) -> Option<&'a Section> {
        if index >= lesson.sections.len() {
            return None;
        }
        return Some(&lesson.sections[index]);
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
    pub sections: Vec<Section>,
}

#[derive(Debug, Deserialize)]
pub struct Section {
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
