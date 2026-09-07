use substring::Substring;

use crate::kana_converter::RomanjiToKanaConverter;
use crate::kanji_converter::HiragaToKanjiConverter;
use crate::lessons::Book;

use std::collections::HashSet;

pub enum CurrentScreen {
    Welcome,
    Chat,
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

    pub chat: usize,

    pub lesson_idx: usize,
    pub section_idx: Option<usize>,
    pub phrase_idx: usize,
    pub translation_direction: TranslationDirection,

    pub prev_section_idx: Option<usize>,
    pub prev_phrase_idx: Option<usize>,
    pub prev_translation_direction: Option<TranslationDirection>,
    pub prev_answer: Option<String>,

    pub kanji_offset: usize,

    pub randomize_section: bool,
    pub asked_questions: Vec<HashSet<usize>>,
}

impl Context {
    fn new() -> Self {
        Context {
            current_screen: CurrentScreen::Welcome,
            current_selection: CurrentSelection::Lesson,
            chat: 0,
            lesson_idx: 0,
            section_idx: None,
            phrase_idx: 0,
            translation_direction: TranslationDirection::ToEN,
            prev_section_idx: None,
            prev_phrase_idx: None,
            prev_translation_direction: None,
            prev_answer: None,
            kanji_offset: 0,
            randomize_section: false,
            asked_questions: Vec::<HashSet<usize>>::new(),
        }
    }
}

#[derive(PartialEq)]
pub struct Offset {
    pub pos: usize,
    pub len: usize,
}

impl Offset {
    fn new(pos: usize, len: usize) -> Self {
        Offset { pos, len }
    }
}

struct OffsetMap {
    src: Offset,
    dest: Offset,
}

impl OffsetMap {
    fn new(src: Offset, dest: Offset) -> Self {
        OffsetMap { src, dest }
    }
}

enum HilightDirection {
    None,
    Left,
    Right,
}

struct Cursor {
    offset: Offset,
    highlight_dir: HilightDirection,
}

impl Cursor {
    fn new() -> Self {
        Cursor {
            offset: Offset::new(0, 0),
            highlight_dir: HilightDirection::None,
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

    kana_offsets: Vec<OffsetMap>,

    pub highlighted_kanji: Vec<char>,
    pub kana_offset: usize,
    pub kana_len: usize,
    kanji_offsets: Vec<(usize, usize, usize)>,

    cursor: Cursor,

    pub debug: String,
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
            kana_offsets: Vec::new(),
            highlighted_kanji: Vec::new(),
            kana_offset: 0,
            kana_len: 1,
            kanji_offsets: Vec::new(),
            cursor: Cursor::new(),
            debug: String::from("debug: "),
        }
    }

    pub fn get_cursor_pos(&self) -> &Offset {
        return &self.cursor.offset;
    }

    pub fn cursor_right(&mut self) {
        if let Some(offset) = self
            .kana_offsets
            .iter()
            .find(|&offset| self.cursor.offset.pos + self.cursor.offset.len == offset.dest.pos)
        {
            // move cursor right
            self.cursor.offset.pos = offset.dest.pos;
            self.cursor.offset.len = offset.dest.len;
        } else if self.cursor.offset.pos + self.cursor.offset.len < self.kana.chars().count() {
            // move cursor right for non kana chars
            self.cursor.offset.pos += self.cursor.offset.len;
            self.cursor.offset.len = 1;
        } else if let Some(offset) = self.kana_offsets.iter().find(|&offset| {
            self.cursor.offset.pos + self.cursor.offset.len == offset.dest.pos + offset.dest.len
        }) {
            // at end, undo highlighting but keep end char group highlighted
            self.cursor.offset.pos = offset.dest.pos;
            self.cursor.offset.len = offset.dest.len;
        } else if self.cursor.offset.pos + self.cursor.offset.len == self.kana.chars().count() {
            // at end, undo highlighting but keep end non kana char highighted
            self.cursor.offset.pos = self.kana.chars().count() - 1;
            self.cursor.offset.len = 1;
        }
        self.cursor.highlight_dir = HilightDirection::None;
        assert!(self.cursor.offset.len > 0);
        assert!(self.cursor.offset.pos + self.cursor.offset.len <= self.kana.chars().count());
    }

    pub fn cursor_left(&mut self) {
        // update highlighting
        if let Some(offset) = self
            .kana_offsets
            .iter()
            .find(|&offset| self.cursor.offset.pos == offset.dest.pos + offset.dest.len)
        {
            // move cursor left
            self.cursor.offset.pos = offset.dest.pos;
            self.cursor.offset.len = offset.dest.len;
        } else if self.cursor.offset.pos > 0 {
            self.cursor.offset.pos -= 1;
            self.cursor.offset.len = 1;
        } else if let Some(offset) = self
            .kana_offsets
            .iter()
            .find(|&offset| self.cursor.offset.pos == offset.dest.pos)
        {
            // at beginning, undo highlighting but keep first char group highlighted
            self.cursor.offset.pos = offset.dest.pos;
            self.cursor.offset.len = offset.dest.len;
        } else if self.cursor.offset.pos == 0 {
            // at beginning, undo highlighting but keep first char highlighted
            self.cursor.offset.len = 1;
        }
        self.cursor.highlight_dir = HilightDirection::None;
        assert!(self.cursor.offset.len > 0);
        assert!(self.cursor.offset.pos + self.cursor.offset.len <= self.kana.chars().count());
    }

    pub fn cursor_highlight_right(&mut self) {
        match self.cursor.highlight_dir {
            HilightDirection::None | HilightDirection::Right => {
                if let Some(offset) = self.kana_offsets.iter().find(|&offset| {
                    self.cursor.offset.pos + self.cursor.offset.len == offset.dest.pos
                }) {
                    self.cursor.offset.len += offset.dest.len;
                } else if self.cursor.offset.pos + self.cursor.offset.len
                    < self.kana.chars().count()
                {
                    self.cursor.offset.len += 1;
                }
                self.cursor.highlight_dir = HilightDirection::Right;
            }
            HilightDirection::Left => {
                if let Some(offset) = self
                    .kana_offsets
                    .iter()
                    .find(|&offset| self.cursor.offset.pos == offset.dest.pos)
                {
                    self.cursor.offset.pos += offset.dest.len;
                    self.cursor.offset.len -= offset.dest.len;
                    if let Some(cur_offset) = self
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos == offset.dest.pos + offset.dest.len);
                        self.cursor.highlight_dir = HilightDirection::None;
                    }
                } else if self.cursor.offset.len > 0 {
                    self.cursor.offset.pos += 1;
                    self.cursor.offset.len -= 1;
                }
            }
        }
        assert!(self.cursor.offset.len > 0);
        assert!(self.cursor.offset.pos + self.cursor.offset.len <= self.kana.chars().count());
    }

    pub fn cursor_highlight_left(&mut self) {
        match self.cursor.highlight_dir {
            HilightDirection::Right => {
                if let Some(offset) = self.kana_offsets.iter().find(|&offset| {
                    self.cursor.offset.pos + self.cursor.offset.len
                        == offset.dest.pos + offset.dest.len
                }) {
                    self.cursor.offset.len -= offset.dest.len;
                    if let Some(cur_offset) = self
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.cursor.highlight_dir = HilightDirection::None;
                    }
                } else if self.cursor.offset.len > 0 {
                    self.cursor.offset.len -= 1;
                    // highlighting a single element, might need to switch directions
                    if self.cursor.offset.len == 1 {
                        self.cursor.highlight_dir = HilightDirection::None;
                    }
                }
            }
            HilightDirection::None | HilightDirection::Left => {
                if let Some(offset) = self
                    .kana_offsets
                    .iter()
                    .find(|&offset| self.cursor.offset.pos == offset.dest.pos + offset.dest.len)
                {
                    self.cursor.offset.pos = offset.dest.pos;
                    self.cursor.offset.len += offset.dest.len;
                } else if self.cursor.offset.pos > 0 {
                    {
                        self.cursor.offset.pos -= 1;
                        self.cursor.offset.len += 1;
                    }
                    self.cursor.highlight_dir = HilightDirection::Left;
                }
            }
        }
        assert!(self.cursor.offset.len > 0);
        assert!(self.cursor.offset.pos + self.cursor.offset.len <= self.kana.chars().count());
    }

    pub fn push_char(&mut self, value: char) {
        self.romanji.push(value);

        let end = self.romanji.chars().count();
        let start: usize;
        {
            // the longest kana conversion is 4 chars, so we use a moving window of 4 chars
            let gap: usize;
            if let Some(last_offset) = self.kana_offsets.last() {
                gap = end - (last_offset.src.pos + last_offset.src.len)
            } else {
                gap = end
            }
            if gap > 4 {
                start = end - 4;
            } else {
                start = end - gap;
            }
        }

        let mut cursor_len = 1;
        let mut is_converted = false;
        for i in start..end {
            let romanji_substr = self.romanji.substring(i, end);
            // conversion successful
            if let Some(converted_str) = self.kana_converter.convert(romanji_substr, true) {
                is_converted = true;

                // remove unconverted string from end of output string
                if let Some((byte_idx, _)) = self
                    .kana
                    .char_indices()
                    .nth(self.kana.chars().count().saturating_sub(end - i - 1))
                {
                    self.kana.truncate(byte_idx);
                }

                // update converted string
                self.kana_offsets.push(OffsetMap::new(
                    Offset::new(i, end - i),
                    Offset::new(self.kana.chars().count(), converted_str.chars().count()),
                ));
                self.kana.push_str(&converted_str.to_string());
                cursor_len = converted_str.chars().count();
                break;
            }
        }

        if !is_converted {
            self.kana.push(value);
        }

        self.cursor.offset.pos = self.kana.chars().count() - cursor_len;
        self.cursor.offset.len = cursor_len;
    }

    pub fn pop_char(&mut self) {
        // check if we removed kana, if so then add back unconverted chars
        if let Some(last_offset) = self.kana_offsets.last()
            && self.romanji.chars().count() == last_offset.src.pos + last_offset.src.len
        {
            // remove romanji
            if let Some((byte_idx, _)) = self.romanji.char_indices().nth(
                self.romanji
                    .chars()
                    .count()
                    .saturating_sub(last_offset.src.len),
            ) {
                self.romanji.truncate(byte_idx);
            }

            // remove kana
            if let Some((byte_idx, _)) = self.kana.char_indices().nth(
                self.kana
                    .chars()
                    .count()
                    .saturating_sub(last_offset.dest.len),
            ) {
                self.kana.truncate(byte_idx);
            }
            self.kana_offsets.pop();
        } else {
            self.romanji.pop();
            self.kana.pop();
        }

        // update cursor
        if self.kana.chars().count() != 0 {
            // cursor touching last kana char
            if let Some(last_offset) = self.kana_offsets.last()
                && self.romanji.chars().count() == last_offset.src.pos + last_offset.src.len
            {
                self.cursor.offset.len = last_offset.dest.len;
            } else {
                self.cursor.offset.len = 1;
            }
            self.cursor.offset.pos = self.kana.chars().count() - self.cursor.offset.len;
        }
    }

    pub fn push_kanji_offset(&mut self, offset: (usize, usize, usize)) {
        let start = offset.0;
        let end = offset.0 + offset.1;
        let kanji_list_offset = offset.2;
        assert!(start < self.kana.chars().count() && end <= self.kana.chars().count());
        let kana_substr: String = self.kana.chars().take(end).skip(start).collect();
        let kanji_list = self.kanji_converter.convert(&kana_substr);
        if kanji_list_offset < kanji_list.len() {
            // if exact match, undo matching
            if let Some(index) = self
                .kanji_offsets
                .iter()
                .position(|&kanji_offset| kanji_offset == offset)
            {
                self.kanji_offsets.remove(index);
            } else {
                // remove colliding offsets
                self.kanji_offsets.retain(|&kanji_offset| {
                    let start = offset.0;
                    let end = offset.0 + offset.1;
                    return !(start >= kanji_offset.0 && start < kanji_offset.0 + kanji_offset.1)
                        && !(end > kanji_offset.0 && end <= kanji_offset.0 + kanji_offset.1);
                });
                self.kanji_offsets.push(offset);
            }
            self.highlighted_kanji = kanji_list;
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

            assert!(
                self.kana_offset < self.kana.chars().count()
                    && self.kana_offset + self.kana_len <= self.kana.chars().count()
            );
            let kana_substr: String = self
                .kana
                .chars()
                .take(self.kana_offset + self.kana_len)
                .skip(self.kana_offset)
                .collect();
            self.highlighted_kanji = self.kanji_converter.convert(&kana_substr);
        } else {
            self.highlighted_kanji.clear();
        }
    }

    pub fn get_romanji(&self) -> &String {
        return &self.romanji;
    }

    pub fn get_kana(&self) -> &String {
        return &self.kana;
    }

    pub fn get_kanji(&self) -> &String {
        return &self.kanji;
    }

    pub fn reset_keyboard(&mut self) {
        self.romanji.clear();
        self.kana.clear();
        self.kanji.clear();
    }
}
