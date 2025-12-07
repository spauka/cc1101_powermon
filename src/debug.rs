use core::fmt::Write;
use flipperzero::println;
use heapless::String;

static RX_LOG_BUF_LEN: usize = 3 * 128;

fn debug_buf(rx_buf: &[u8; 128], read_bytes: usize) {
    let mut hex_line: String<RX_LOG_BUF_LEN> = String::new();
    for (idx, byte) in rx_buf[0..read_bytes].iter().enumerate() {
        if idx > 0 {
            let _ = write!(hex_line, " ");
        }
        let _ = write!(hex_line, "{:02X}", byte);
    }
    println!("rx_buf = {}", hex_line.as_str());
}
