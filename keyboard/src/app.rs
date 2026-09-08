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

pub struct App {
    pub book: Book,
    kana_converter: RomanjiToKanaConverter,
    kanji_converter: HiragaToKanjiConverter,
    pub context: Context,

    romanji: String,
    kana: String,
    kanji: String,

    merge_kana: Option<MergeKana>,
    kana_to_kanji: Option<KanaToKanji>,

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
            merge_kana: None,
            kana_to_kanji: None,
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
        } else {
            unreachable!();
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
        } else {
            unreachable!();
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
                    if let Some(cur_offset) = self
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.cursor.highlight_dir = HilightDirection::None;
                    } else {
                        self.cursor.highlight_dir = HilightDirection::Right;
                    }
                } else if self.cursor.offset.pos + self.cursor.offset.len
                    < self.kana.chars().count()
                {
                    self.cursor.offset.len += 1;
                    if self.cursor.offset.pos == self.kana.chars().count() - 1 {
                        // cant highlight right, still neutral
                        self.cursor.highlight_dir = HilightDirection::None;
                    } else {
                        self.cursor.highlight_dir = HilightDirection::Right;
                    }
                }
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
                    } else if self.cursor.offset.len == 1 {
                        self.cursor.highlight_dir = HilightDirection::None;
                    }
                } else if self.cursor.offset.len > 0 {
                    self.cursor.offset.pos += 1;
                    self.cursor.offset.len -= 1;
                    if self
                        .kana_offsets
                        .iter()
                        .any(|offset| self.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        //assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.cursor.highlight_dir = HilightDirection::None;
                    } else if self.cursor.offset.len == 1 {
                        // cant highlght right, still neutral
                        self.cursor.highlight_dir = HilightDirection::None;
                    }
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
                    } else if self.cursor.offset.len == 1 {
                        self.cursor.highlight_dir = HilightDirection::None;
                    }
                } else if self.cursor.offset.len > 0 {
                    self.cursor.offset.len -= 1;
                    // highlighting a single element, might need to switch directions
                    if self
                        .kana_offsets
                        .iter()
                        .any(|offset| self.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        //assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.cursor.highlight_dir = HilightDirection::None;
                    } else if self.cursor.offset.len == 1 {
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
                    if let Some(cur_offset) = self
                        .kana_offsets
                        .iter()
                        .find(|&offset| self.cursor.offset == offset.dest)
                    {
                        // on current position, switch highlightdirection
                        assert!(cur_offset.dest.pos + cur_offset.dest.len == offset.dest.pos);
                        self.cursor.highlight_dir = HilightDirection::None;
                    } else {
                        self.cursor.highlight_dir = HilightDirection::Left;
                    }
                } else if self.cursor.offset.pos > 0 {
                    self.cursor.offset.pos -= 1;
                    self.cursor.offset.len += 1;
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
        self.cursor.highlight_dir = HilightDirection::None;
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
        self.cursor.highlight_dir = HilightDirection::None;
    }

    pub fn check_merge_kana(&mut self) {
        self.merge_kana = None;
        if let Some(start_offset) = self
            .kana_offsets
            .iter()
            .find(|&offset| self.cursor.offset.pos == offset.dest.pos)
            && let Some(end_offset) = self.kana_offsets.iter().find(|&offset| {
                self.cursor.offset.pos + self.cursor.offset.len == offset.dest.pos + offset.dest.len
            })
        {
            // make sure theyre contiguous, only support merging of contiguous kana
            if start_offset.dest.pos + start_offset.dest.len == end_offset.dest.pos {
                let romanji_offset = Offset::new(
                    start_offset.src.pos,
                    start_offset.src.len + end_offset.src.len,
                );
                let romanji_substr: &str = self
                    .romanji
                    .substring(romanji_offset.pos, romanji_offset.pos + romanji_offset.len);
                if let Some(merge_kana) = self.kana_converter.convert(romanji_substr, true) {
                    assert!(self.kana_to_kanji.is_none());
                    let len: usize = merge_kana.chars().count();
                    self.merge_kana = Some(MergeKana::new(
                        merge_kana,
                        OffsetMap::new(romanji_offset, Offset::new(start_offset.dest.pos, len)),
                    ));
                }
            }
        }
    }

    pub fn update_merge_kana(&mut self) {
        if let Some(merge_kana) = &self.merge_kana {
            assert!(self.kana_to_kanji.is_none());
            // remove first part of contiguous merge offsets
            self.kana_offsets
                .retain(|offset| offset.dest.pos != merge_kana.offset_map.dest.pos);
            // remove second part of contiguous merge offsets
            self.kana_offsets.retain(|offset| {
                offset.dest.pos + offset.dest.len
                    != merge_kana.offset_map.dest.pos + merge_kana.offset_map.dest.len
            });
            // update offset positions when merged kana is shorter than original
            assert!(self.cursor.offset.len >= merge_kana.offset_map.dest.len);
            let offset_diff = self.cursor.offset.len - merge_kana.offset_map.dest.len;
            if offset_diff > 0 {
                for offset in &mut self.kana_offsets {
                    if merge_kana.offset_map.dest.pos + merge_kana.offset_map.dest.len
                        <= offset.dest.pos
                    {
                        offset.dest.pos -= offset_diff;
                    }
                }
            }
            self.kana_offsets.push(merge_kana.offset_map);
            self.kana_offsets
                .sort_by_key(|offset_map| offset_map.src.pos);
            let start_byte_index = self
                .kana
                .char_indices()
                .nth(self.cursor.offset.pos)
                .map(|(i, _)| i)
                .unwrap_or(self.kana.len());
            let end_byte_index = self
                .kana
                .char_indices()
                .nth(self.cursor.offset.pos + self.cursor.offset.len)
                .map(|(i, _)| i)
                .unwrap_or(self.kana.len());
            self.kana.replace_range(
                start_byte_index..end_byte_index,
                &merge_kana.kana.to_string(),
            );
            // update cursor
            self.cursor.offset = merge_kana.offset_map.dest;
            self.cursor.highlight_dir = HilightDirection::None;
        }
        self.merge_kana = None;
    }

    pub fn check_kana_to_kanji(&mut self) {
        if let Some(start_offset) = self
            .kana_offsets
            .iter()
            .find(|&offset| self.cursor.offset.pos == offset.dest.pos)
            && let Some(end_offset) = self.kana_offsets.iter().find(|&offset| {
                self.cursor.offset.pos + self.cursor.offset.len == offset.dest.pos + offset.dest.len
            })
        {
            let romanji_offset = Offset::new(
                start_offset.src.pos,
                end_offset.src.pos + end_offset.src.len - start_offset.src.pos,
            );

            // keep existing conversion if the same
            if let Some(existing_kana_to_kanji) = &self.kana_to_kanji
                && existing_kana_to_kanji.offset_map.src != romanji_offset
            {
                self.kana_to_kanji = None;
            }

            if self.kana_to_kanji.is_none() {
                let kana_substr: &str = self.kana.substring(
                    self.cursor.offset.pos,
                    self.cursor.offset.pos + self.cursor.offset.len,
                );
                let kanji = self.kanji_converter.convert(kana_substr);
                if !kanji.is_empty() {
                    assert!(self.merge_kana.is_none());
                    self.kana_to_kanji = Some(KanaToKanji::new(
                        SelectKanji::new(kanji, 0),
                        OffsetMap::new(romanji_offset, Offset::new(start_offset.dest.pos, 1)),
                    ));
                }
            }
        } else {
            self.kana_to_kanji = None;
        }
    }

    pub fn convert_kana_to_kanji(&mut self) {
        if let Some(kana_to_kanji) = &self.kana_to_kanji {
            assert!(self.merge_kana.is_none());
            // remove offsets that are replaces ny new kanji offse
            self.kana_offsets.retain(|offset| {
                !(offset.dest.pos >= self.cursor.offset.pos
                    && offset.dest.pos + offset.src.len
                        <= self.cursor.offset.pos + self.cursor.offset.len)
            });
            // update offset positions when merged kana is shorter than original
            assert!(self.cursor.offset.len >= kana_to_kanji.offset_map.dest.len);
            let offset_diff = self.cursor.offset.len - kana_to_kanji.offset_map.dest.len;
            if offset_diff > 0 {
                for offset in &mut self.kana_offsets {
                    if kana_to_kanji.offset_map.dest.pos + kana_to_kanji.offset_map.dest.len
                        <= offset.dest.pos
                    {
                        offset.dest.pos -= offset_diff;
                    }
                }
            }
            self.kana_offsets.push(kana_to_kanji.offset_map);
            self.kana_offsets
                .sort_by_key(|offset_map| offset_map.src.pos);
            let start_byte_index = self
                .kana
                .char_indices()
                .nth(self.cursor.offset.pos)
                .map(|(i, _)| i)
                .unwrap_or(self.kana.len());
            let end_byte_index = self
                .kana
                .char_indices()
                .nth(self.cursor.offset.pos + self.cursor.offset.len)
                .map(|(i, _)| i)
                .unwrap_or(self.kana.len());
            let mut buf = [0; 4];
            let kanji_char: &str =
                kana_to_kanji.kanji.kanji_list[kana_to_kanji.kanji.offset].encode_utf8(&mut buf);
            self.kana
                .replace_range(start_byte_index..end_byte_index, kanji_char);
            // update cursor
            self.cursor.offset = kana_to_kanji.offset_map.dest;
            self.cursor.highlight_dir = HilightDirection::None;
        }
        self.kana_to_kanji = None;
    }

    pub fn kanji_select_down(&mut self) {
        if let Some(kana_to_kanji) = &mut self.kana_to_kanji
            && kana_to_kanji.kanji.offset < kana_to_kanji.kanji.kanji_list.len() - 1
        {
            kana_to_kanji.kanji.offset += 1;
        }
    }

    pub fn kanji_select_up(&mut self) {
        if let Some(kana_to_kanji) = &mut self.kana_to_kanji
            && kana_to_kanji.kanji.offset > 0
        {
            kana_to_kanji.kanji.offset -= 1;
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

    pub fn get_merge_kana(&self) -> Option<&String> {
        if let Some(merge_kana) = &self.merge_kana {
            return Some(&merge_kana.kana);
        }
        return None;
    }

    pub fn get_kana_to_kanji(&self) -> Option<&SelectKanji> {
        if let Some(kana_to_kanji) = &self.kana_to_kanji {
            return Some(&kana_to_kanji.kanji);
        }
        return None;
    }

    pub fn reset_keyboard(&mut self) {
        self.romanji.clear();
        self.kana.clear();
        self.kanji.clear();
    }
}
