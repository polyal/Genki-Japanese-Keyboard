mod cli;
mod kana_converter;
mod kanji_converter;
mod lessons;

use cli::Reviewer;

fn main() {
    cli();
}

fn cli() {
    let reviewer = Reviewer::new();
    reviewer.start();
}
