use dioxus::prelude::*;

// I fucking LOVE Action Message Format
struct AMFReader {
    buffer: Vec<u8>,
    read_head: usize,
    out: Vec<SyntaxByte>,
    encoding: u8,
}

impl AMFReader {
    pub fn new(buffer: &Vec<u8>) -> Self {
        AMFReader {
            buffer: buffer.clone(),
            read_head: 0,
            out: Vec::new(),
            encoding: 0,
        }
    }

    pub fn highlight(&mut self) {
        while &self.read_head < &self.buffer.len() {
            if self.encoding == 0 {
                self.decode_amf0();
            } else {
                self.decode_amf3();
            }
        }
    }

    pub fn decode_amf0(&mut self) {
        let current_byte = self.read_byte();
        match current_byte {
            0x00 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id: 2,
                    color: "text-ctp-blue",
                });
            }
            0x01 => {}
            0x02 => {}
            0x06 => {}
            0x07 => {}
            0x08 => {}
            0x09 => {}
            0x0A => {}
            0x0B => {}
            0x0C => {}
            0x0D => {}
            0x0F => {}
            0x10 => {}
            0x11 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id: 1,
                    color: "text-ctp-pink",
                });
                self.encoding = 3;
            }
            _ => self.out.push(SyntaxByte {
                value: current_byte,
                object_id: 0,
                color: "text-ctp-red",
            }),
        }
    }
    pub fn decode_amf3(&mut self) {
        let current_byte = self.read_byte();
        match current_byte {
            _ => self.out.push(SyntaxByte {
                value: current_byte,
                object_id: 1,
                color: "text-ctp-green",
            }),
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        let b = self.buffer[self.read_head];
        self.read_head += 1;
        b
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
