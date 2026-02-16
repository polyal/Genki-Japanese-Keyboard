mod cli;
mod kana_converter;
mod kanji_converter;
mod lessons;

use cli::Reviewer;
use kana_converter::RomanjiToKanaConverter;
use kanji_converter::HiragaToKanjiConverter;

fn main() {
    cli();
}

fn cli() {
    let reviewer = Reviewer::new();
    reviewer.start();
}
