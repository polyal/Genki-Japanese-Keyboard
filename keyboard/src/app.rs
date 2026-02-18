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

pub struct Context {
    pub current_screen: CurrentScreen,
    pub current_selection: CurrentSelection,
    pub lesson: usize,
    pub section: usize,
}

impl Context {
    fn new() -> Self {
        Context {
            current_screen: CurrentScreen::Welcome,
            current_selection: CurrentSelection::Lesson,
            lesson: 0,
            section: 0,
        }
    }
}

pub struct App {
    pub book: Book,
    kana_converter: RomanjiToKanaConverter,
    kanji_converter: HiragaToKanjiConverter,
    pub context: Context,

    romanji: String,
    kana: String,
    kanji: String,

    pub kana_offset: usize,
    pub kana_len: usize,
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

    pub fn get_romanji(&self) -> String {
        return self.romanji.clone();
    }

    pub fn get_kana(&self) -> String {
        return self.kana.clone();
    }
}
