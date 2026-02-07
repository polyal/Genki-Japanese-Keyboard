mod converter;

use converter::RomanjiToKanaConverter;
use std::io;

fn main() {
  let mut buffer = String::new();
  io::stdin().read_line(&mut buffer).expect("failed to read line");
  buffer.pop(); // remove '\n'

  let mut converter = RomanjiToKanaConverter::new();
  let kana = converter.convert(&buffer);
  println!("converted '{buffer}' -> '{kana}'");
}
