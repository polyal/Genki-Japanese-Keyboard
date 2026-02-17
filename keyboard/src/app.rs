use crate::kana_converter::RomanjiToKanaConverter;
use crate::kanji_converter::HiragaToKanjiConverter;
use crate::lessons::{Book, Lesson, Phrase, Section};

pub enum CurrentScreen {
    Welcome,
    LessonSelect,
    Review,
}

pub struct App {
    book: Book,
    kana_converter: RomanjiToKanaConverter,
    kanji_converter: HiragaToKanjiConverter,
    pub current_screen: CurrentScreen,
}

impl App {
    pub fn new() -> Self {
        App {
            book: Book::new(),
            kana_converter: RomanjiToKanaConverter::new(),
            kanji_converter: HiragaToKanjiConverter::new(),
            current_screen: CurrentScreen::Welcome,
        }
    }
}
