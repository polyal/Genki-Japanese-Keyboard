use std::io;

fn main() {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("failed to read line");

    println!("read then wrote: {buffer}");
}
