use std::io;

fn main() {
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer);

    println!("read then wrote: {}", buffer);
}
