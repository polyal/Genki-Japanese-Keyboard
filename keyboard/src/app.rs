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
}

impl App {
    pub fn new() -> Self {
        App {
            book: Book::new(),
            kana_converter: RomanjiToKanaConverter::new(),
            kanji_converter: HiragaToKanjiConverter::new(),
            context: Context::new(),
        }
    }
}
