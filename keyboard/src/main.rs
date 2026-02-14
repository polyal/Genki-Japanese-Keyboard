mod kana_converter;
mod kanji_converter;
mod lessons;

use kana_converter::RomanjiToKanaConverter;
use kanji_converter::HiragaToKanjiConverter;
use lessons::Reviewer;
use std::io;

fn main() {
    let mut buffer = String::new();
    println!("!! Genki-Japanese-Keyboard !!\n");
    println!("[0] Convert");
    println!("[-] Study");

    io::stdin()
        .read_line(&mut buffer)
        .expect("failed to read line");
    buffer.pop(); // remove '\n'
    if buffer == "0" {
        loop {
            println!("\nEnter something to convert: ");

            buffer.clear();
            io::stdin()
                .read_line(&mut buffer)
                .expect("failed to read line");
            buffer.pop(); // remove '\n'
            if buffer == ":q" {
                break;
            }
            let kana_converter = RomanjiToKanaConverter::new();
            let kana = kana_converter.convert(&buffer);
            println!("converted '{buffer}' -> '{kana}'");

            let kanji_converter = HiragaToKanjiConverter::new();
            let kanji = kanji_converter.convert(&kana);
            for kanji_char in kanji {
                println!("converted '{kana}' -> '{kanji_char}'");
            }
        }
    } else {
        let reviewer = Reviewer::new();
        reviewer.start();
    }
}
