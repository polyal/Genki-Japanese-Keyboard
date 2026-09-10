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

#[derive(Copy, Clone, PartialEq)]
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
            randomize_section: false,
            asked_questions: Vec::<HashSet<usize>>::new(),
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct Offset {
    pub pos: usize,
    pub len: usize,
}

impl Offset {
    fn new(pos: usize, len: usize) -> Self {
        Offset { pos, len }
    }
}

#[derive(Clone, Copy)]
struct OffsetMap {
    src: Offset,
    dest: Offset,
}

impl OffsetMap {
    fn new(src: Offset, dest: Offset) -> Self {
        OffsetMap { src, dest }
    }
}

struct MergeKana {
    kana: String,
    offset_map: OffsetMap,
}

impl MergeKana {
    fn new(kana: String, offset_map: OffsetMap) -> Self {
        MergeKana { kana, offset_map }
    }
}

pub struct SelectKanji {
    pub kanji_list: Vec<char>,
    pub offset: usize,
}

impl SelectKanji {
    fn new(kanji_list: Vec<char>, offset: usize) -> Self {
        SelectKanji { kanji_list, offset }
    }
}

struct KanaToKanji {
    kanji: SelectKanji,
    offset_map: OffsetMap,
}

impl KanaToKanji {
    fn new(kanji: SelectKanji, offset_map: OffsetMap) -> Self {
        KanaToKanji { kanji, offset_map }
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

struct Keyboard {
    kana_converter: RomanjiToKanaConverter,
    kanji_converter: HiragaToKanjiConverter,

    romanji: String,
    kana: String,

    kana_offsets: Vec<OffsetMap>,

    merge_kana: Option<MergeKana>,
    kana_to_kanji: Option<KanaToKanji>,

    english: bool,

    cursor: Cursor,
}

impl Keyboard {
    fn new() -> Self {
        Keyboard {
            kana_converter: RomanjiToKanaConverter::new(),
            kanji_converter: HiragaToKanjiConverter::new(),
            romanji: String::new(),
            kana: String::new(),
            kana_offsets: Vec::new(),
            merge_kana: None,
            kana_to_kanji: None,
            english: false,
            cursor: Cursor::new(),
        }
    }
}

pub struct App {
    pub book: Book,
    pub context: Context,
    keyboard: Keyboard,

    pub debug: String,
}

impl App {
    pub fn new() -> Self {
        App {
            book: Book::new(),
            context: Context::new(),
            keyboard: Keyboard::new(),
            debug: String::from("debug: "),
        }
    }

    pub fn get_cursor_pos(&self) -> &Offset {
        return &self.keyboard.cursor.offset;
    }

    pub fn cursor_right(&mut self) {
        if let Some(offset) = self.keyboard.kana_offsets.iter().find(|&offset| {
            self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len == offset.dest.pos
        }) {
            // move cursor right
            self.keyboard.cursor.offset.pos = offset.dest.pos;
            self.keyboard.cursor.offset.len = offset.dest.len;
        } else if self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
            < self.keyboard.kana.chars().count()
        {
            // move cursor right for non kana chars
            self.keyboard.cursor.offset.pos += self.keyboard.cursor.offset.len;
            self.keyboard.cursor.offset.len = 1;
        } else if let Some(offset) = self.keyboard.kana_offsets.iter().find(|&offset| {
            self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                == offset.dest.pos + offset.dest.len
        }) {
            // at end, undo highlighting but keep end char group highlighted
            self.keyboard.cursor.offset.pos = offset.dest.pos;
            self.keyboard.cursor.offset.len = offset.dest.len;
        } else if self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
            == self.keyboard.kana.chars().count()
        {
            // at end, undo highlighting but keep end non kana char highighted
            self.keyboard.cursor.offset.pos = self.keyboard.kana.chars().count() - 1;
            self.keyboard.cursor.offset.len = 1;
        } else {
            unreachable!();
        }
        self.keyboard.cursor.highlight_dir = HilightDirection::None;
        assert!(self.keyboard.cursor.offset.len > 0);
        assert!(
            self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                <= self.keyboard.kana.chars().count()
        );
    }

    pub fn cursor_left(&mut self) {
        // update highlighting
        if let Some(offset) =
            self.keyboard.kana_offsets.iter().find(|&offset| {
                self.keyboard.cursor.offset.pos == offset.dest.pos + offset.dest.len
            })
        {
            // move cursor left
            self.keyboard.cursor.offset.pos = offset.dest.pos;
            self.keyboard.cursor.offset.len = offset.dest.len;
        } else if self.keyboard.cursor.offset.pos > 0 {
            self.keyboard.cursor.offset.pos -= 1;
            self.keyboard.cursor.offset.len = 1;
        } else if let Some(offset) = self
            .keyboard
            .kana_offsets
            .iter()
            .find(|&offset| self.keyboard.cursor.offset.pos == offset.dest.pos)
        {
            // at beginning, undo highlighting but keep first char group highlighted
            self.keyboard.cursor.offset.pos = offset.dest.pos;
            self.keyboard.cursor.offset.len = offset.dest.len;
        } else if self.keyboard.cursor.offset.pos == 0 {
            // at beginning, undo highlighting but keep first char highlighted
            self.keyboard.cursor.offset.len = 1;
        } else {
            unreachable!();
        }
        self.keyboard.cursor.highlight_dir = HilightDirection::None;
        assert!(self.keyboard.cursor.offset.len > 0);
        assert!(
            self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                <= self.keyboard.kana.chars().count()
        );
    }

    pub fn cursor_highlight_right(&mut self) {
        match self.keyboard.cursor.highlight_dir {
            HilightDirection::None | HilightDirection::Right => {
                if let Some(offset) = self.keyboard.kana_offsets.iter().find(|&offset| {
                    self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                        == offset.dest.pos
                }) {
                    self.keyboard.cursor.offset.len += offset.dest.len;
                    if let Some(cur_offset) = self
                        .keyboard
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.keyboard.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    } else {
                        self.keyboard.cursor.highlight_dir = HilightDirection::Right;
                    }
                } else if self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                    < self.keyboard.kana.chars().count()
                {
                    self.keyboard.cursor.offset.len += 1;
                    if self.keyboard.cursor.offset.pos == self.keyboard.kana.chars().count() - 1 {
                        // cant highlight right, still neutral
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    } else {
                        self.keyboard.cursor.highlight_dir = HilightDirection::Right;
                    }
                }
            }
            HilightDirection::Left => {
                if let Some(offset) = self
                    .keyboard
                    .kana_offsets
                    .iter()
                    .find(|&offset| self.keyboard.cursor.offset.pos == offset.dest.pos)
                {
                    self.keyboard.cursor.offset.pos += offset.dest.len;
                    self.keyboard.cursor.offset.len -= offset.dest.len;
                    if let Some(cur_offset) = self
                        .keyboard
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.keyboard.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos == offset.dest.pos + offset.dest.len);
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    } else if self.keyboard.cursor.offset.len == 1 {
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    }
                } else if self.keyboard.cursor.offset.len > 0 {
                    self.keyboard.cursor.offset.pos += 1;
                    self.keyboard.cursor.offset.len -= 1;
                    if self
                        .keyboard
                        .kana_offsets
                        .iter()
                        .any(|offset| self.keyboard.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        //assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    } else if self.keyboard.cursor.offset.len == 1 {
                        // cant highlght right, still neutral
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    }
                }
            }
        }
        assert!(self.keyboard.cursor.offset.len > 0);
        assert!(
            self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                <= self.keyboard.kana.chars().count()
        );
    }

    pub fn cursor_highlight_left(&mut self) {
        match self.keyboard.cursor.highlight_dir {
            HilightDirection::Right => {
                if let Some(offset) = self.keyboard.kana_offsets.iter().find(|&offset| {
                    self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                        == offset.dest.pos + offset.dest.len
                }) {
                    self.keyboard.cursor.offset.len -= offset.dest.len;
                    if let Some(cur_offset) = self
                        .keyboard
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.keyboard.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    } else if self.keyboard.cursor.offset.len == 1 {
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    }
                } else if self.keyboard.cursor.offset.len > 0 {
                    self.keyboard.cursor.offset.len -= 1;
                    // highlighting a single element, might need to switch directions
                    if self
                        .keyboard
                        .kana_offsets
                        .iter()
                        .any(|offset| self.keyboard.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        //assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    } else if self.keyboard.cursor.offset.len == 1 {
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    }
                }
            }
            HilightDirection::None | HilightDirection::Left => {
                if let Some(offset) = self.keyboard.kana_offsets.iter().find(|&offset| {
                    self.keyboard.cursor.offset.pos == offset.dest.pos + offset.dest.len
                }) {
                    self.keyboard.cursor.offset.pos = offset.dest.pos;
                    self.keyboard.cursor.offset.len += offset.dest.len;
                    if let Some(cur_offset) = self
                        .keyboard
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.keyboard.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.keyboard.cursor.highlight_dir = HilightDirection::None;
                    } else {
                        self.keyboard.cursor.highlight_dir = HilightDirection::Left;
                    }
                } else if self.keyboard.cursor.offset.pos > 0 {
                    self.keyboard.cursor.offset.pos -= 1;
                    self.keyboard.cursor.offset.len += 1;
                    self.keyboard.cursor.highlight_dir = HilightDirection::Left;
                }
            }
        }
        assert!(self.keyboard.cursor.offset.len > 0);
        assert!(
            self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                <= self.keyboard.kana.chars().count()
        );
    }

    pub fn push_char(&mut self, value: char) {
        let mut cursor_len = 1;
        let mut is_converted = false;

        self.keyboard.romanji.push(value);
        if !self.keyboard.english {
            let end = self.keyboard.romanji.chars().count();
            let start: usize;
            {
                // the longest kana conversion is 4 chars, so we use a moving window of 4 chars
                let gap: usize;
                if let Some(last_offset) = self.keyboard.kana_offsets.last() {
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

            for i in start..end {
                let romanji_substr = self.keyboard.romanji.substring(i, end);
                // conversion successful
                if let Some(converted_str) =
                    self.keyboard.kana_converter.convert(romanji_substr, true)
                {
                    is_converted = true;

                    // remove unconverted string from end of output string
                    if let Some((byte_idx, _)) = self.keyboard.kana.char_indices().nth(
                        self.keyboard
                            .kana
                            .chars()
                            .count()
                            .saturating_sub(end - i - 1),
                    ) {
                        self.keyboard.kana.truncate(byte_idx);
                    }

                    // update converted string
                    self.keyboard.kana_offsets.push(OffsetMap::new(
                        Offset::new(i, end - i),
                        Offset::new(
                            self.keyboard.kana.chars().count(),
                            converted_str.chars().count(),
                        ),
                    ));
                    self.keyboard.kana.push_str(&converted_str.to_string());
                    cursor_len = converted_str.chars().count();
                    break;
                }
            }
        }

        if !is_converted {
            self.keyboard.kana.push(value);
        }
        self.keyboard.cursor.offset.pos = self.keyboard.kana.chars().count() - cursor_len;
        self.keyboard.cursor.offset.len = cursor_len;
        self.keyboard.cursor.highlight_dir = HilightDirection::None;
    }

    pub fn pop_char(&mut self) {
        // check if we removed kana
        if let Some(last_offset) = self.keyboard.kana_offsets.last()
            && self.keyboard.romanji.chars().count() == last_offset.src.pos + last_offset.src.len
        {
            // remove romanji
            if let Some((byte_idx, _)) = self.keyboard.romanji.char_indices().nth(
                self.keyboard
                    .romanji
                    .chars()
                    .count()
                    .saturating_sub(last_offset.src.len),
            ) {
                self.keyboard.romanji.truncate(byte_idx);
            }

            // remove kana
            if let Some((byte_idx, _)) = self.keyboard.kana.char_indices().nth(
                self.keyboard
                    .kana
                    .chars()
                    .count()
                    .saturating_sub(last_offset.dest.len),
            ) {
                self.keyboard.kana.truncate(byte_idx);
            }
            self.keyboard.kana_offsets.pop();
        } else {
            self.keyboard.romanji.pop();
            self.keyboard.kana.pop();
        }

        // update cursor
        if self.keyboard.kana.chars().count() != 0 {
            // cursor touching last kana char
            if let Some(last_offset) = self.keyboard.kana_offsets.last()
                && self.keyboard.romanji.chars().count()
                    == last_offset.src.pos + last_offset.src.len
            {
                self.keyboard.cursor.offset.len = last_offset.dest.len;
            } else {
                self.keyboard.cursor.offset.len = 1;
            }
            self.keyboard.cursor.offset.pos =
                self.keyboard.kana.chars().count() - self.keyboard.cursor.offset.len;
        }
        self.keyboard.cursor.highlight_dir = HilightDirection::None;
    }

    pub fn check_merge_kana(&mut self) -> bool {
        self.keyboard.merge_kana = None;
        if let Some(start_offset) = self
            .keyboard
            .kana_offsets
            .iter()
            .find(|&offset| self.keyboard.cursor.offset.pos == offset.dest.pos)
            && let Some(end_offset) = self.keyboard.kana_offsets.iter().find(|&offset| {
                self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                    == offset.dest.pos + offset.dest.len
            })
        {
            // make sure theyre contiguous, only support merging of contiguous kana
            if start_offset.dest.pos + start_offset.dest.len == end_offset.dest.pos {
                let romanji_offset = Offset::new(
                    start_offset.src.pos,
                    start_offset.src.len + end_offset.src.len,
                );
                let romanji_substr: &str = self
                    .keyboard
                    .romanji
                    .substring(romanji_offset.pos, romanji_offset.pos + romanji_offset.len);
                if let Some(merge_kana) = self.keyboard.kana_converter.convert(romanji_substr, true)
                {
                    let len: usize = merge_kana.chars().count();
                    self.keyboard.merge_kana = Some(MergeKana::new(
                        merge_kana,
                        OffsetMap::new(romanji_offset, Offset::new(start_offset.dest.pos, len)),
                    ));
                }
            }
            return true;
        }
        return false;
    }

    pub fn update_merge_kana(&mut self) -> bool {
        if let Some(merge_kana) = &self.keyboard.merge_kana {
            assert!(self.keyboard.kana_to_kanji.is_none());
            // remove first part of contiguous merge offsets
            self.keyboard
                .kana_offsets
                .retain(|offset| offset.src.pos != merge_kana.offset_map.src.pos);
            // remove second part of contiguous merge offsets
            self.keyboard.kana_offsets.retain(|offset| {
                offset.src.pos + offset.src.len
                    != merge_kana.offset_map.src.pos + merge_kana.offset_map.src.len
            });
            // update offset positions when merged kana is shorter than original
            assert!(self.keyboard.cursor.offset.len >= merge_kana.offset_map.dest.len);
            let offset_diff = self.keyboard.cursor.offset.len - merge_kana.offset_map.dest.len;
            if offset_diff > 0 {
                for offset in &mut self.keyboard.kana_offsets {
                    if merge_kana.offset_map.dest.pos + merge_kana.offset_map.dest.len
                        <= offset.dest.pos
                    {
                        offset.dest.pos -= offset_diff;
                    }
                }
            }
            self.keyboard.kana_offsets.push(merge_kana.offset_map);
            self.keyboard
                .kana_offsets
                .sort_by_key(|offset_map| offset_map.src.pos);
            let start_byte_index = self
                .keyboard
                .kana
                .char_indices()
                .nth(self.keyboard.cursor.offset.pos)
                .map(|(i, _)| i)
                .unwrap_or(self.keyboard.kana.len());
            let end_byte_index = self
                .keyboard
                .kana
                .char_indices()
                .nth(self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len)
                .map(|(i, _)| i)
                .unwrap_or(self.keyboard.kana.len());
            self.keyboard.kana.replace_range(
                start_byte_index..end_byte_index,
                &merge_kana.kana.to_string(),
            );
            // update cursor
            self.keyboard.cursor.offset = merge_kana.offset_map.dest;
            self.keyboard.cursor.highlight_dir = HilightDirection::None;
            self.keyboard.merge_kana = None;
            return true;
        }
        return false;
    }

    pub fn check_kana_to_kanji(&mut self) -> bool {
        if let Some(start_offset) = self
            .keyboard
            .kana_offsets
            .iter()
            .find(|&offset| self.keyboard.cursor.offset.pos == offset.dest.pos)
            && let Some(end_offset) = self.keyboard.kana_offsets.iter().find(|&offset| {
                self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len
                    == offset.dest.pos + offset.dest.len
            })
        {
            let romanji_offset = Offset::new(
                start_offset.src.pos,
                end_offset.src.pos + end_offset.src.len - start_offset.src.pos,
            );

            // keep existing conversion if the same
            if let Some(existing_kana_to_kanji) = &self.keyboard.kana_to_kanji
                && existing_kana_to_kanji.offset_map.src != romanji_offset
            {
                self.keyboard.kana_to_kanji = None;
            }

            if self.keyboard.kana_to_kanji.is_none() {
                let kana_substr: &str = self.keyboard.kana.substring(
                    self.keyboard.cursor.offset.pos,
                    self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len,
                );
                let kanji = self.keyboard.kanji_converter.convert(kana_substr);
                if !kanji.is_empty() {
                    self.keyboard.kana_to_kanji = Some(KanaToKanji::new(
                        SelectKanji::new(kanji, 0),
                        OffsetMap::new(romanji_offset, Offset::new(start_offset.dest.pos, 1)),
                    ));
                }
            }
            return true;
        }
        self.keyboard.kana_to_kanji = None;
        return false;
    }

    pub fn convert_kana_to_kanji(&mut self) -> bool {
        if let Some(kana_to_kanji) = &self.keyboard.kana_to_kanji {
            assert!(self.keyboard.merge_kana.is_none());
            // remove offsets that are replaced by new kanji offse
            self.keyboard.kana_offsets.retain(|offset| {
                !(offset.dest.pos >= self.keyboard.cursor.offset.pos
                    && offset.dest.pos + offset.dest.len
                        <= self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len)
            });
            // update offset positions when merged kana is shorter than original
            assert!(self.keyboard.cursor.offset.len >= kana_to_kanji.offset_map.dest.len);
            let offset_diff = self.keyboard.cursor.offset.len - kana_to_kanji.offset_map.dest.len;
            if offset_diff > 0 {
                for offset in &mut self.keyboard.kana_offsets {
                    if kana_to_kanji.offset_map.dest.pos + kana_to_kanji.offset_map.dest.len
                        <= offset.dest.pos
                    {
                        offset.dest.pos -= offset_diff;
                    }
                }
            }
            self.keyboard.kana_offsets.push(kana_to_kanji.offset_map);
            self.keyboard
                .kana_offsets
                .sort_by_key(|offset_map| offset_map.src.pos);
            let start_byte_index = self
                .keyboard
                .kana
                .char_indices()
                .nth(self.keyboard.cursor.offset.pos)
                .map(|(i, _)| i)
                .unwrap_or(self.keyboard.kana.len());
            let end_byte_index = self
                .keyboard
                .kana
                .char_indices()
                .nth(self.keyboard.cursor.offset.pos + self.keyboard.cursor.offset.len)
                .map(|(i, _)| i)
                .unwrap_or(self.keyboard.kana.len());
            let mut buf = [0; 4];
            let kanji_char: &str =
                kana_to_kanji.kanji.kanji_list[kana_to_kanji.kanji.offset].encode_utf8(&mut buf);
            self.keyboard
                .kana
                .replace_range(start_byte_index..end_byte_index, kanji_char);
            // update cursor
            self.keyboard.cursor.offset = kana_to_kanji.offset_map.dest;
            self.keyboard.cursor.highlight_dir = HilightDirection::None;
            self.keyboard.kana_to_kanji = None;
            return true;
        }
        return false;
    }

    pub fn kanji_select_down(&mut self) {
        if let Some(kana_to_kanji) = &mut self.keyboard.kana_to_kanji
            && kana_to_kanji.kanji.offset < kana_to_kanji.kanji.kanji_list.len() - 1
        {
            kana_to_kanji.kanji.offset += 1;
        }
    }

    pub fn kanji_select_up(&mut self) {
        if let Some(kana_to_kanji) = &mut self.keyboard.kana_to_kanji
            && kana_to_kanji.kanji.offset > 0
        {
            kana_to_kanji.kanji.offset -= 1;
        }
    }

    pub fn toggle_english(&mut self) {
        self.keyboard.english = !self.keyboard.english;
    }

    pub fn set_english(&mut self, val: bool) {
        self.keyboard.english = val;
    }

    pub fn get_romanji(&self) -> &String {
        return &self.keyboard.romanji;
    }

    pub fn get_kana(&self) -> &String {
        return &self.keyboard.kana;
    }

    pub fn get_merge_kana(&self) -> Option<&String> {
        if let Some(merge_kana) = &self.keyboard.merge_kana {
            return Some(&merge_kana.kana);
        }
        return None;
    }

    pub fn get_kana_to_kanji(&self) -> Option<&SelectKanji> {
        if let Some(kana_to_kanji) = &self.keyboard.kana_to_kanji {
            return Some(&kana_to_kanji.kanji);
        }
        return None;
    }

    pub fn get_use_english(&self) -> bool {
        return self.keyboard.english;
    }

    pub fn reset_conversion_selection(&mut self) {
        self.keyboard.merge_kana = None;
        self.keyboard.kana_to_kanji = None;
    }

    pub fn reset_keyboard(&mut self) {
        self.keyboard.romanji.clear();
        self.keyboard.kana.clear();
        self.keyboard.merge_kana = None;
        self.keyboard.kana_to_kanji = None;
        self.keyboard.english = false;
        self.keyboard.kana_offsets.clear();
    }
}
