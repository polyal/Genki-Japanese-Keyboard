mod converter;
mod lessons;

use converter::RomanjiToKanaConverter;
use lessons::Reviewer;
use std::io;


fn main() {
  let mut buffer = String::new();
  println!("!! Genki-Japanese-Keyboard !!\n");
  println!("[0] Convert");
  println!("[-] Study");

  io::stdin().read_line(&mut buffer).expect("failed to read line");
  buffer.pop(); // remove '\n'
  if buffer == "0" {
    loop {
      println!("\nEnter something to convert: ");

      buffer.clear();
      io::stdin().read_line(&mut buffer).expect("failed to read line");
      buffer.pop(); // remove '\n'
      if buffer == ":q" {
        break;
      }
      let converter = RomanjiToKanaConverter::new();
      let kana = converter.convert(&buffer);
      println!("converted '{buffer}' -> '{kana}'");
    }
  }
  else {
    let reviewer = Reviewer::new();
    reviewer.start();
  }
}
