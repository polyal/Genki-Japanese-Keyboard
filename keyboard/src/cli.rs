use rand::Rng;
use std::collections::HashSet;
use std::io;

use crate::kana_converter::RomanjiToKanaConverter;
use crate::kanji_converter::HiragaToKanjiConverter;
use crate::lessons::Book;
use crate::lessons::Lesson;
use crate::lessons::Phrase;
use crate::lessons::Section;

const COMMAND_QUIT: &str = ":q";
const COMMAND_BACK: &str = ":b";

pub struct Reviewer {
    book: Book,
    kana_converter: RomanjiToKanaConverter,
    kanji_converter: HiragaToKanjiConverter,
}

impl Reviewer {
    pub fn new() -> Self {
        Reviewer {
            book: Book::new(),
            kana_converter: RomanjiToKanaConverter::new(),
            kanji_converter: HiragaToKanjiConverter::new(),
        }
    }

    pub fn start(&self) {
        let mut buffer = String::new();
        println!("!! Genki-Japanese-Keyboard !!\n");
        println!("[0] Convert");
        println!("[-] Study");

        Self::get_user_input(&mut buffer);
        if buffer == "0" {
            self.convert();
        } else {
            self.study();
        }
    }

    fn convert(&self) {
        let mut buffer = String::new();
        loop {
            println!("\nEnter something to convert: ");

            Self::get_user_input(&mut buffer);
            if buffer == COMMAND_QUIT {
                break;
            }
            let kana = self.kana_converter.convert(&buffer);
            println!("converted '{buffer}' -> '{kana}'");

            let kanji = self.kanji_converter.convert(&kana);
            for kanji_char in kanji {
                println!("converted '{kana}' -> '{kanji_char}'");
            }
        }
    }

    fn study(&self) {
        let mut buffer = String::new();
        let lessons = self.book.get_lessons();
        while buffer != COMMAND_QUIT {
            buffer.clear();
            // pick lesson
            let mut lesson_idx = usize::MAX;
            loop {
                if let Some(lesson) = Book::get_lesson(lessons, lesson_idx) {
                    let mut section_idx: usize;
                    loop {
                        // test section
                        println!("\nPick a section: ");
                        Self::print_sections(&lesson);
                        Self::get_user_input(&mut buffer);
                        match buffer.parse::<usize>() {
                            Ok(n) => section_idx = n,
                            Err(_e) => {
                                if buffer == COMMAND_BACK {
                                    break;
                                } else {
                                    continue;
                                }
                            }
                        }

                        if let Some(section) = Book::get_section(lesson, section_idx) {
                            self.review_section(section);
                        } else {
                            self.review_lesson(lesson);
                        }
                    }
                }

                // lesson selection
                println!("\nPick a lesson: ");
                Self::print_lessons(lessons);
                Self::get_user_input(&mut buffer);
                match buffer.parse::<usize>() {
                    Ok(n) => lesson_idx = n,
                    Err(_e) => break,
                }
            }
        }
    }

    fn get_user_input(buffer: &mut String) {
        buffer.clear();
        io::stdin().read_line(buffer).expect("failed to read line");
        buffer.pop(); // remove '\n'
    }

    fn print_lessons(lessons: &Vec<Lesson>) {
        for lesson in lessons {
            println!(
                "  [{}] {} - {}",
                lesson.index, lesson.name_en, lesson.name_jp
            );
        }
    }

    fn print_sections(lesson: &Lesson) {
        let sections = Book::get_sections(lesson);
        let mut index: usize = 0;
        for section in sections {
            println!("  [{index}] {}", section.name);
            index += 1;
        }
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

    fn review_section(&self, section: &Section) {
        let mut asked: HashSet<usize> = HashSet::new();
        loop {
            if asked.len() == section.phrases.len() {
                asked.clear();
            }
            let mut phrase_idx = rand::thread_rng().gen_range(0..section.phrases.len());
            while !asked.insert(phrase_idx) {
                phrase_idx = rand::thread_rng().gen_range(0..section.phrases.len());
            }

            print!("\n  [{}/{}] ", phrase_idx, section.phrases.len(),);
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
            } else {
                println!("translate '{}' to english", phrase.jp);
            }

            Self::get_user_input(&mut buffer);
            if buffer == COMMAND_BACK {
                return false;
            }

            println!("  your    answer: '{}'", buffer);
            println!("  correct answer: '{}'", phrase.en);
        } else {
            println!("translate '{}' to japanese", phrase.en);
            Self::get_user_input(&mut buffer);
            if buffer == COMMAND_BACK {
                return false;
            }

            let kana = self.kana_converter.convert(&buffer);
            println!("  your    answer: '{}'", kana);
            if let Some(kanji) = &phrase.kanji {
                println!("  correct answer: '{}' - '{}'", phrase.jp, kanji);
            } else {
                println!("  correct answer: '{}'", phrase.jp);
            }
        }
        return true;
    }
}
