use crate::{kana_converter::RomajiToKanaConverter, kanji_converter::HiragaToKanjiConverter};

const MAX_VALID_ROMAJI_LEN: usize = 4;

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

#[derive(PartialEq, Clone, Copy)]
struct OffsetMap {
    src: Offset,
    dest: Offset,
}

impl OffsetMap {
    fn new(src: Offset, dest: Offset) -> Self {
        OffsetMap { src, dest }
    }
}

pub struct SelectKanji {
    pub kanji_list: Vec<char>,
    pub idx: usize,
}

impl SelectKanji {
    fn new(kanji_list: Vec<char>, idx: usize) -> Self {
        SelectKanji { kanji_list, idx }
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
            offset: Offset::new(0, 1),
            highlight_dir: HilightDirection::None,
        }
    }
}

pub struct Keyboard {
    kana_converter: RomajiToKanaConverter,
    kanji_converter: HiragaToKanjiConverter,

    romaji: String,
    kana: String,

    kana_offsets: Vec<OffsetMap>,

    kana_to_kanji: Option<KanaToKanji>,

    english: bool,

    cursor: Cursor,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            kana_converter: RomajiToKanaConverter::new(),
            kanji_converter: HiragaToKanjiConverter::new(),
            romaji: String::from(' '),
            kana: String::from(' '),
            kana_offsets: Vec::new(),
            kana_to_kanji: None,
            english: false,
            cursor: Cursor::new(),
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
        let mut is_converted = false;

        let get_last_offset = || -> Option<OffsetMap> {
            let mut last_offset: Option<OffsetMap> = None;
            for offset in self.kana_offsets.iter() {
                if offset.dest.pos < self.cursor.offset.pos {
                    last_offset = Some(*offset);
                }
            }
            return last_offset;
        };

        // get insert position
        let romaji_offset: Offset;
        let last_offset = get_last_offset();
        if let Some(last_offset) = last_offset {
            let unconverted_len =
                self.cursor.offset.pos - (last_offset.dest.pos + last_offset.dest.len);
            let romaji_pos = last_offset.src.pos + last_offset.src.len + unconverted_len;
            romaji_offset = Offset::new(romaji_pos, 1);
        } else {
            romaji_offset = Offset::new(self.cursor.offset.pos, 1);
        }
        assert!(romaji_offset.pos < self.romaji.chars().count());
        assert!(self.cursor.offset.pos < self.kana.chars().count());
        self.romaji.insert(romaji_offset.pos, value);

        if !self.english {
            let mut try_merge_kana: bool;
            let end = romaji_offset.pos + romaji_offset.len;
            let start: usize;
            {
                // look behind to see if we can convert unconverted romaji
                let gap: usize;
                if let Some(last_offset) = last_offset {
                    // check if last_offset is an n conversion so we can auto merge conversion
                    // eg. ni -> に not んい
                    // because n/N gets converted to ん/ン at previous insertion
                    if last_offset.src.len == 1
                        && self
                            .romaji
                            .chars()
                            .nth(last_offset.src.pos)
                            .unwrap()
                            .eq_ignore_ascii_case(&'n')
                    {
                        gap = end - last_offset.src.pos;
                        try_merge_kana = true;
                    } else {
                        try_merge_kana = false;
                        gap = end - (last_offset.src.pos + last_offset.src.len);
                    }
                } else {
                    try_merge_kana = false;
                    gap = end;
                }
                // the longest kana conversion is 4 chars, so we look back a max of 4 chars
                if gap > MAX_VALID_ROMAJI_LEN {
                    start = end - MAX_VALID_ROMAJI_LEN;
                } else {
                    start = end - gap;
                }
            }

            // try look back sliding window conversion
            for i in start..end {
                if i != start {
                    try_merge_kana = false;
                }

                let romaji_substr = self.romaji.get(i..end).unwrap();
                if let Some(converted_str) = self.kana_converter.convert(romaji_substr, true) {
                    // conversion successful
                    is_converted = true;

                    // update converted string
                    let romaji_len = romaji_substr.chars().count() - 1;
                    let start_idx = self.cursor.offset.pos - romaji_len;
                    let end_idx = self.cursor.offset.pos;
                    let start_byte_index = self
                        .kana
                        .char_indices()
                        .nth(start_idx)
                        .map(|(i, _)| i)
                        .unwrap();
                    let end_byte_index = self
                        .kana
                        .char_indices()
                        .nth(end_idx)
                        .map(|(i, _)| i)
                        .unwrap();
                    self.kana.replace_range(
                        start_byte_index..end_byte_index,
                        &converted_str.to_string(),
                    );

                    // update offset positions after conversion
                    if romaji_len > converted_str.chars().count() {
                        assert!(!try_merge_kana);
                        let len_diff = romaji_len - converted_str.chars().count();
                        for offset in &mut self.kana_offsets {
                            if self.cursor.offset.pos <= offset.dest.pos {
                                offset.dest.pos -= len_diff;
                                offset.src.pos += 1;
                            }
                        }
                    } else if romaji_len <= converted_str.chars().count() {
                        // even if len diff is 0, the src.pos still has to be updated
                        let len_diff = converted_str.chars().count() - romaji_len;
                        for offset in &mut self.kana_offsets {
                            if try_merge_kana && last_offset.unwrap().dest == offset.dest {
                                // we if did an n merge conversion then just update existing offset
                                offset.dest.len = converted_str.chars().count();
                                offset.src.len = end - i;
                            } else if self.cursor.offset.pos <= offset.dest.pos {
                                offset.dest.pos += len_diff;
                                offset.src.pos += 1
                            }
                        }
                    }

                    // already updated existing offset above
                    if !try_merge_kana {
                        // insert new offset
                        self.kana_offsets.push(OffsetMap::new(
                            Offset::new(i, end - i),
                            Offset::new(start_idx, converted_str.chars().count()),
                        ));
                        self.kana_offsets.sort_by_key(|offset| offset.src.pos);
                    }
                    self.cursor.offset.pos = start_idx + converted_str.chars().count();
                    break;
                }
            }
        }

        if !is_converted {
            let insert_idx = self
                .kana
                .char_indices()
                .nth(self.cursor.offset.pos)
                .map(|(idx, _)| idx)
                .unwrap();
            self.kana.insert(insert_idx, value);

            // update offsets passed cursor to account for insersion of new char
            for offset in &mut self.kana_offsets {
                if self.cursor.offset.pos <= offset.dest.pos {
                    offset.dest.pos += 1;
                    offset.src.pos += 1
                }
            }

            self.cursor.offset.pos += 1;
        }
        self.cursor.highlight_dir = HilightDirection::None;
    }

    pub fn pop_char(&mut self) {
        if self.cursor.offset.pos == 0 {
            self.cursor.highlight_dir = HilightDirection::None;
            return;
        }

        let split_kana: bool;
        let split_kana_offsets: Option<(OffsetMap, OffsetMap)>;
        let src_diff_size: usize;
        let dest_diff_size: usize;
        // check if we need to remove kana
        if let Some(last_idx) = self
            .kana_offsets
            .iter()
            .position(|&offset| self.cursor.offset.pos == offset.dest.pos + offset.dest.len)
        {
            // remove last offset
            let last_offset = self.kana_offsets.remove(last_idx);
            assert!(last_offset.src.pos + last_offset.src.len < self.romaji.chars().count());
            assert!(last_offset.dest.pos + last_offset.dest.len < self.kana.chars().count());

            let start_byte_index = self
                .kana
                .char_indices()
                .nth(last_offset.dest.pos)
                .map(|(i, _)| i)
                .unwrap();
            let end_byte_index = self
                .kana
                .char_indices()
                .nth(last_offset.dest.pos + last_offset.dest.len)
                .map(|(i, _)| i)
                .unwrap();
            let remove_kana = &self.kana[start_byte_index..end_byte_index];

            // check if last_offset is a merge kana conversion so we can split conversion instead of delete
            // eg. に -> んい
            let merge_kana = [
                "な", "に", "ぬ", "ね", "の", "にゃ", "にゅ", "にょ", "ナ", "ニ", "ヌ", "ネ", "ノ",
                "ニャ", "ニュ", "ニョ",
            ];
            if merge_kana.contains(&remove_kana) {
                split_kana = true;

                // redo conversion for split
                let n_substr = self
                    .romaji
                    .get(last_offset.src.pos..last_offset.src.pos + 1)
                    .unwrap();
                let r_substr = self
                    .romaji
                    .get(last_offset.src.pos + 1..last_offset.src.pos + last_offset.src.len)
                    .unwrap();
                let n_split = self.kana_converter.convert(n_substr, true).unwrap();
                let r_split = self.kana_converter.convert(r_substr, true).unwrap();
                let combined = n_split + &r_split;

                // update newly split string
                self.kana
                    .replace_range(start_byte_index..end_byte_index, &combined);

                assert!(r_substr.chars().count() == last_offset.src.len - 1);
                assert!(r_split.chars().count() == combined.chars().count() - 1);
                split_kana_offsets = Some((
                    OffsetMap::new(
                        Offset::new(last_offset.src.pos, 1),
                        Offset::new(last_offset.dest.pos, 1),
                    ),
                    OffsetMap::new(
                        Offset::new(last_offset.src.pos + 1, r_substr.chars().count()),
                        Offset::new(last_offset.dest.pos + 1, r_split.chars().count()),
                    ),
                ));

                assert!(combined.chars().count() >= last_offset.dest.len);
                src_diff_size = 0;
                dest_diff_size = combined.chars().count() - last_offset.dest.len;
            } else {
                split_kana = false;
                split_kana_offsets = None;

                // remove romaji
                self.romaji.replace_range(
                    last_offset.src.pos..last_offset.src.pos + last_offset.src.len,
                    "",
                );

                // remove kana
                self.kana
                    .replace_range(start_byte_index..end_byte_index, "");

                src_diff_size = last_offset.src.len;
                dest_diff_size = last_offset.dest.len;
            }
        } else {
            split_kana = false;
            split_kana_offsets = None;

            // get insert position
            let romaji_offset: Offset;
            {
                let mut last_offset: Option<&OffsetMap> = None;
                for offset in self.kana_offsets.iter() {
                    if offset.dest.pos < self.cursor.offset.pos {
                        last_offset = Some(offset);
                    }
                }

                if let Some(last_offset) = last_offset {
                    let gap =
                        self.cursor.offset.pos - (last_offset.dest.pos + last_offset.dest.len);
                    let romaji_pos = last_offset.src.pos + last_offset.src.len + gap;
                    romaji_offset = Offset::new(romaji_pos, 1);
                } else {
                    romaji_offset = Offset::new(self.cursor.offset.pos, 1);
                }
            }
            assert!(romaji_offset.pos < self.romaji.chars().count());
            assert!(self.cursor.offset.pos < self.kana.chars().count());

            self.romaji
                .replace_range(romaji_offset.pos - 1..romaji_offset.pos, "");

            let start_idx = self.cursor.offset.pos - 1;
            let end_idx: usize = self.cursor.offset.pos;
            let start_byte_index = self
                .kana
                .char_indices()
                .nth(start_idx)
                .map(|(i, _)| i)
                .unwrap();
            let end_byte_index = self
                .kana
                .char_indices()
                .nth(end_idx)
                .map(|(i, _)| i)
                .unwrap();
            self.kana
                .replace_range(start_byte_index..end_byte_index, "");

            src_diff_size = 1;
            dest_diff_size = 1;
        }

        // update offsets
        if !split_kana {
            for offset in &mut self.kana_offsets {
                if self.cursor.offset.pos <= offset.dest.pos {
                    offset.src.pos -= src_diff_size;
                    offset.dest.pos -= dest_diff_size;
                }
            }
        } else {
            assert!(src_diff_size == 0);
            for offset in &mut self.kana_offsets {
                if self.cursor.offset.pos <= offset.dest.pos {
                    offset.dest.pos += dest_diff_size;
                }
            }

            // push offset for newly split kana
            assert!(split_kana_offsets.is_some());
            self.kana_offsets.push(split_kana_offsets.unwrap().0);
            self.kana_offsets.push(split_kana_offsets.unwrap().1);
            self.kana_offsets.sort_by_key(|offset| offset.src.pos);
        }

        // update cursor
        if split_kana {
            self.cursor.offset.pos += dest_diff_size;
        } else {
            self.cursor.offset.pos -= dest_diff_size;
        }
        self.cursor.highlight_dir = HilightDirection::None;
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
            let romaji_offset = Offset::new(
                start_offset.src.pos,
                end_offset.src.pos + end_offset.src.len - start_offset.src.pos,
            );

            // keep existing conversion if the same
            if let Some(existing_kana_to_kanji) = &self.kana_to_kanji
                && existing_kana_to_kanji.offset_map.src != romaji_offset
            {
                self.kana_to_kanji = None;
            }

            if self.kana_to_kanji.is_none() {
                let kana_substr: String = self
                    .kana
                    .chars()
                    .skip(self.cursor.offset.pos)
                    .take(self.cursor.offset.len)
                    .collect();
                let kanji = self.kanji_converter.convert(&kana_substr.to_string());
                if !kanji.is_empty() {
                    self.kana_to_kanji = Some(KanaToKanji::new(
                        SelectKanji::new(kanji, 0),
                        OffsetMap::new(romaji_offset, Offset::new(start_offset.dest.pos, 1)),
                    ));
                }
            }
        } else {
            self.kana_to_kanji = None;
        }
    }

    pub fn convert_kana_to_kanji(&mut self) -> bool {
        if let Some(kana_to_kanji) = &self.kana_to_kanji {
            // remove offsets that are replaced by new kanji offse
            self.kana_offsets.retain(|offset| {
                !(offset.dest.pos >= self.cursor.offset.pos
                    && offset.dest.pos + offset.dest.len
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
                .unwrap();
            let end_byte_index = self
                .kana
                .char_indices()
                .nth(self.cursor.offset.pos + self.cursor.offset.len)
                .map(|(i, _)| i)
                .unwrap();
            let mut buf = [0; 4];
            let kanji_char: &str =
                kana_to_kanji.kanji.kanji_list[kana_to_kanji.kanji.idx].encode_utf8(&mut buf);
            self.kana
                .replace_range(start_byte_index..end_byte_index, kanji_char);
            // update cursor
            self.cursor.offset = kana_to_kanji.offset_map.dest;
            self.cursor.highlight_dir = HilightDirection::None;
            self.kana_to_kanji = None;
            return true;
        }
        return false;
    }

    pub fn kanji_select_down(&mut self) {
        if let Some(kana_to_kanji) = &mut self.kana_to_kanji
            && kana_to_kanji.kanji.idx < kana_to_kanji.kanji.kanji_list.len() - 1
        {
            kana_to_kanji.kanji.idx += 1;
        }
    }

    pub fn kanji_select_up(&mut self) {
        if let Some(kana_to_kanji) = &mut self.kana_to_kanji
            && kana_to_kanji.kanji.idx > 0
        {
            kana_to_kanji.kanji.idx -= 1;
        }
    }

    pub fn toggle_english(&mut self) {
        self.english = !self.english;
    }

    pub fn set_english(&mut self, val: bool) {
        self.english = val;
    }

    pub fn get_romaji(&self) -> &str {
        return &self.romaji;
    }

    pub fn get_kana(&self) -> &str {
        return &self.kana;
    }

    pub fn get_kana_to_kanji(&self) -> Option<&SelectKanji> {
        if let Some(kana_to_kanji) = &self.kana_to_kanji {
            return Some(&kana_to_kanji.kanji);
        }
        return None;
    }

    pub fn get_use_english(&self) -> bool {
        return self.english;
    }

    pub fn reset_conversion_selection(&mut self) {
        self.kana_to_kanji = None;
    }

    pub fn reset(&mut self) {
        self.romaji.clear();
        self.kana.clear();
        self.romaji.push(' ');
        self.kana.push(' ');
        self.kana_to_kanji = None;
        self.english = false;
        self.kana_offsets.clear();
        self.cursor.offset.pos = 0;
        self.cursor.offset.len = 1;
    }
}
