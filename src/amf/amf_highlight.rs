use dioxus::prelude::*;

// I fucking LOVE Action Message Format
struct AMFReader {
    buffer: Vec<u8>,
    read_head: usize,
    out: Vec<SyntaxByte>,
}

impl AMFReader {
    pub fn new(buffer: &Vec<u8>) -> Self {
        AMFReader {
            buffer: buffer.clone(),
            read_head: 0,
            out: Vec::new(),
        }
    }

    pub fn highlight(&mut self) {
        while &self.read_head < &self.buffer.len() {
            let current_byte = self.buffer[self.read_head];
            if current_byte == 0x09 {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id: 10,
                    color: "text-ctp-green",
                });
            } else {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id: 0,
                    color: "text-ctp-text",
                });
            }

            self.read_head += 1;
        }
    }
}

struct SyntaxByte {
    value: u8,
    object_id: usize,
    color: &'static str,
}

#[component]
pub fn highlight_bytes(buffer: Vec<u8>) -> Element {
    let mut reader = AMFReader::new(&buffer);
    reader.highlight();
    rsx! {
        for byte in reader.out {
            span {
                class: "{byte.color} hex",
                id: "{byte.object_id}",
                "{byte.value:02x} "
            }
        }
    }
}
