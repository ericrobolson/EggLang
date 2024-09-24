mod cat;
mod packet;
fn main() {}

fn lsp_main() {
    // Read in stdin until eof and print it back out
    let mut buffer = String::new();

    // Loop, adding stdin to buffer until EOF
    while let Ok(n) = std::io::stdin().read_line(&mut buffer) {
        if n == 0 {
            // Decode packet
            match packet::read_packet(&buffer) {
                Ok(packet) => {
                    println!("Decoded: {}", packet.decoded);
                    println!("Remaining: {}", packet.remaining);
                }
                Err(e) => {
                    println!("Error decoding packet: {}", e);
                    buffer = String::new();
                }
            }
        }
    }
}
