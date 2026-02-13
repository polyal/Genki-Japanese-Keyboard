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
    println!("Enter something to convert: ");

    buffer.clear();
    io::stdin().read_line(&mut buffer).expect("failed to read line");
    buffer.pop(); // remove '\n'
    let mut converter = RomanjiToKanaConverter::new();
    let kana = converter.convert(&buffer);
    println!("converted '{buffer}' -> '{kana}'");
  }
  else {
    let reviewer = Reviewer::new();
    while buffer != "exit" {
      buffer.clear();
      // pick lesson
      let mut lesson_idx = usize::MAX;
      loop {
        if let Some(lesson) = &reviewer.get_lesson(lesson_idx) {
          let mut section_idx: usize;
          loop {
            // test section
            println!("\nPick a section: ");
            reviewer.print_sections(&lesson);   

            buffer.clear();
            io::stdin().read_line(&mut buffer).expect("failed to read line");
            buffer.pop(); // remove '\n'
            match buffer.parse::<usize>() {
              Ok(n) => section_idx = n,
              Err(_e) => break,
            }
            if let Some(section) = &reviewer.get_section(lesson, section_idx) {
              reviewer.review_section(section);
            }
            else {
              reviewer.review_lesson(lesson);
            }
          }
        }

        // lesson selection
        println!("\nPick a lesson: ");
        reviewer.print_lessons();

        buffer.clear();
        io::stdin().read_line(&mut buffer).expect("failed to read line");
        buffer.pop(); // remove '\n'
        match buffer.parse::<usize>() {
          Ok(n) => lesson_idx = n,
          Err(_e) => break,
        }
      }
    }
  }
}
