use crate::kana_converter::RomanjiToKanaConverter;
use crate::kanji_converter::HiragaToKanjiConverter;
use crate::lessons::{Book, Lesson, Phrase, Section};

pub enum CurrentScreen {
    Welcome,
    LessonSelect,
    Review,
}

pub enum CurrentSelection {
    Lesson,
    Section,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TranslationDirection {
    ToEN,
    ToJP,
}

pub struct Context {
    pub current_screen: CurrentScreen,
    pub current_selection: CurrentSelection,
    pub lesson: usize,
    pub section: usize,
    pub phrase: usize,
    pub prev_phrase: Option<usize>,
    pub translation_direction: TranslationDirection,
    pub prev_translation_direction: Option<TranslationDirection>,
    pub prev_answer: Option<String>,
}

impl Context {
    fn new() -> Self {
        Context {
            current_screen: CurrentScreen::Welcome,
            current_selection: CurrentSelection::Lesson,
            lesson: 0,
            section: 0,
            phrase: 0,
            prev_phrase: None,
            translation_direction: TranslationDirection::ToEN,
            prev_translation_direction: None,
            prev_answer: None,
        }
    }
}

pub struct App {
    pub book: Book,
    kana_converter: RomanjiToKanaConverter,
    kanji_converter: HiragaToKanjiConverter,
    pub context: Context,

    pub romanji: String,
    pub kana: String,
    pub kanji: String,

    pub kana_offset: usize,
    pub kana_len: usize,
    kanji_offsets: Vec<(usize, usize, usize)>,
}

impl App {
    pub fn new() -> Self {
        App {
            book: Book::new(),
            kana_converter: RomanjiToKanaConverter::new(),
            kanji_converter: HiragaToKanjiConverter::new(),
            context: Context::new(),
            romanji: String::new(),
            kana: String::new(),
            kanji: String::new(),
            kana_offset: 0,
            kana_len: 1,
            kanji_offsets: Vec::new(),
        }
    }

    pub fn push_char(&mut self, value: char) {
        self.romanji.push(value);
        self.kana = self.kana_converter.convert(&self.romanji);
    }

    pub fn pop_char(&mut self) {
        self.romanji.pop();
        self.kana = self.kana_converter.convert(&self.romanji);
    }

    pub fn push_kanji_offset(&mut self, offset: (usize, usize, usize)) {
        let start = offset.0;
        let end = offset.0 + offset.1;
        let kanji_list_offset = offset.2;
        assert!(start < self.kana.chars().count() && end <= self.kana.chars().count());
        let kana_substr: String = self.kana.chars().take(end).skip(start).collect();
        let kanji_list = self.kanji_converter.convert(&kana_substr);
        if kanji_list_offset < kanji_list.len() {
            // remove colliding offsets
            self.kanji_offsets.retain(|kanji_offset| {
                let start = offset.0;
                let end = offset.0 + offset.1;
                return !(start >= kanji_offset.0 && start < kanji_offset.0 + kanji_offset.1)
                    && !(end > kanji_offset.0 && end <= kanji_offset.0 + kanji_offset.1);
            });
            self.kanji_offsets.push(offset);
        }
    }

    pub fn update_kanji(&mut self) {
        self.kanji = self.kana.clone();
        // remove offsets that no longer exist because of a backspace
        self.kanji_offsets.retain(|kanji_offset| {
            let start = kanji_offset.0;
            let end = kanji_offset.0 + kanji_offset.1;
            return start <= self.kana.chars().count() && end <= self.kana.chars().count();
        });
        // generate kanji from kana and offsets
        if self.kanji.chars().count() > 0 {
            // sort to keep adjusted offset valid
            self.kanji_offsets.sort();
            let mut offset_adjust: usize = 0;
            // update kanji text
            for kanji_offset in &self.kanji_offsets {
                assert!(kanji_offset.1 >= 1);
                let start = kanji_offset.0;
                let end = kanji_offset.0 + kanji_offset.1;
                let kanji_list_offset = kanji_offset.2;
                assert!(start <= self.kana.chars().count() && end <= self.kana.chars().count());
                let kana_substr: String = self.kana.chars().take(end).skip(start).collect();
                let kanji_list = self.kanji_converter.convert(&kana_substr);
                if kanji_list_offset < kanji_list.len() {
                    let kanji_char = String::from(kanji_list[kanji_list_offset]);
                    let start_byte_index = self
                        .kanji
                        .char_indices()
                        .nth(start - offset_adjust)
                        .map(|(i, _)| i)
                        .unwrap_or(self.kanji.len());
                    let end_byte_index = self
                        .kanji
                        .char_indices()
                        .nth(end - offset_adjust)
                        .map(|(i, _)| i)
                        .unwrap_or(self.kanji.len());
                    self.kanji
                        .replace_range(start_byte_index..end_byte_index, &kanji_char);
                    // adjust for shorter len kanji than hiragana after swap
                    offset_adjust += kanji_offset.1 - 1;
                }
            }
        }
    }

    pub fn convert_to_kanji(&self, hiragana: &String) -> Vec<char> {
        return self.kanji_converter.convert(&hiragana);
    }

    pub fn get_romanji(&self) -> String {
        return self.romanji.clone();
    }

    pub fn get_kana(&self) -> String {
        return self.kana.clone();
    }

    pub fn get_kanji(&self) -> String {
        return self.kanji.clone();
    }
}
