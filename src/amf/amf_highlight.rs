use dioxus::html::g::by;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use std::io::Read;

const AMF0_NUMBER: &'static str = "text-ctp-blue";
const AMF0_NUMBER_MARKER: &'static str = "text-ctp-blue/80";
const AMF0_BOOL_TRUE: &'static str = "text-ctp-green/80";
const AMF0_BOOL_TRUE_MARKER: &'static str = "text-ctp-green/80";
const AMF0_BOOL_FALSE: &'static str = "text-ctp-red/80";
const AMF0_BOOL_FALSE_MARKER: &'static str = "text-ctp-red/80";
const AMF0_STRING: &'static str = "text-ctp-yellow";
const AMF0_STRING_MARKER: &'static str = "text-ctp-yellow/80";
const AMF0_OBJECT: &'static str = "text-ctp-teal";
const AMF0_OBJECT_LAYER_TWO: &'static str = "text-ctp-teal/60";
const AMF0_OBJECT_LAYER_THREE: &'static str = "text-ctp-teal/40";
const AMF0_OBJECT_MARKER: &'static str = "text-ctp-teal/80";
const AMF0_NULL: &'static str = "text-ctp-rosewater";
const AMF0_NULL_MARKER: &'static str = "text-ctp-rosewater/80";
const AMF0_TYPED_OBJECT: &'static str = "text-ctp-yellow";
const AMF0_TYPED_OBJECT_MARKER: &'static str = "text-ctp-yellow/80";
const AMF0_SWITCH_MARKER: &'static str = "text-ctp-pink/80";

// I fucking LOVE Action Message Format
struct AMFReader {
    buffer: Vec<u8>,
    read_head: usize,
    out: Vec<SyntaxByte>,
    encoding: u8,
    current_layer: u8, // Change colour depending on object layer
}

impl AMFReader {
    pub fn new(buffer: &Vec<u8>, is_command: bool) -> Self {
        if is_command {
            let encoding = buffer[0];
            AMFReader {
                buffer: buffer.clone(),
                read_head: 1,
                out: Vec::new(),
                encoding,
                current_layer: 0,
            }
        } else {
            AMFReader {
                buffer: buffer.clone(),
                read_head: 0,
                out: Vec::new(),
                encoding: 0,
                current_layer: 0,
            }
        }
    }

    pub fn highlight(&mut self) {
        while &self.read_head < &self.buffer.len() {
            if self.encoding == 0 {
                self.read_amf0();
            } else {
                self.read_amf3();
            }
        }
    }

    fn read_amf0_integer(&mut self) {
        let syntax = SyntaxByte {
            value: 0,
            object_id: 1,
            color: AMF0_NUMBER.parse().unwrap(),
        };
        self.push_bytes(syntax, 7);
    }

    fn read_amf0_bool(&mut self) {
        let byte = self.read_byte();
        let mut syntax = SyntaxByte {
            value: byte,
            object_id: 2,
            color: AMF0_BOOL_TRUE.parse().unwrap(),
        };
        if byte == 0 {
            syntax.color = AMF0_BOOL_FALSE.parse().unwrap();
        }
        self.push_byte(syntax);
    }

    fn read_amf0_string(&mut self) {
        self.read_amf0_utf8(None);
    }

    fn read_amf0_utf_length(&mut self, colour: Option<String>) -> u16 {
        let colour = colour.unwrap_or_else(|| AMF0_STRING.parse().unwrap());
        // I love fighting the borrow checker
        let len_bytes = &self.buffer[self.read_head..self.read_head + 2];
        self.read_head += 2;
        let length = ((len_bytes[0] as u16) << 8) | len_bytes[1] as u16;
        for i in len_bytes {
            let syntax = SyntaxByte {
                value: *i,
                object_id: 3,
                color: colour.clone(),
            };
            self.out.push(syntax);
        }
        length
    }

    fn read_amf0_utf8(&mut self, colour: Option<String>) -> String {
        let length = self.read_amf0_utf_length(colour.clone());
        let colour = colour.unwrap_or_else(|| AMF0_STRING.parse().unwrap());
        let mut out = String::new();
        self.read_bytes(length as usize)
            .read_to_string(&mut out)
            .unwrap();
        for i in out.as_bytes() {
            let syntax = SyntaxByte {
                value: *i,
                object_id: 3,
                color: colour.clone(),
            };
            self.push_byte(syntax);
        }
        out
    }

    fn read_amf0_object(&mut self) {
        self.current_layer += 1;
        loop {
            let key = self.read_amf0_utf8(Some(format!(
                "{}/{}",
                AMF0_OBJECT,
                100 - self.current_layer * 20
            )));
            tracing::debug!("Key: '{}'", key);
            if key.is_empty() {
                let b = self.read_byte();
                let syntax = SyntaxByte {
                    value: b,
                    object_id: 4,
                    color: format!("{}", AMF0_OBJECT_MARKER,),
                };
                self.push_byte(syntax);
                if b == 0x09 {
                    if self.current_layer != 0 {
                        self.current_layer -= 1;
                    }
                    return;
                }
            }
            self.read_amf0();
            tracing::debug!("Next...");
        }
    }

    fn read_amf0_typed_object(&mut self) {
        self.current_layer += 1;
        self.read_amf0_utf8(Some(format!(
            "{}/{}",
            AMF0_TYPED_OBJECT,
            100 - self.current_layer * 20
        )));
        self.read_amf0_object();
    }

    pub fn read_amf0(&mut self) {
        let current_byte = self.read_byte();
        match current_byte {
            0x00 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id: 2,
                    color: AMF0_NUMBER_MARKER.parse().unwrap(),
                });
                self.read_amf0_integer();
            }
            0x01 => {
                let mut syntax = SyntaxByte {
                    value: current_byte,
                    object_id: 2,
                    color: AMF0_BOOL_TRUE_MARKER.parse().unwrap(),
                };
                if current_byte == 0 {
                    syntax.color = AMF0_BOOL_FALSE_MARKER.parse().unwrap();
                }
                self.push_byte(syntax);
                self.read_amf0_bool();
            }
            0x02 => {
                let syntax = SyntaxByte {
                    value: current_byte,
                    object_id: 3,
                    color: AMF0_STRING_MARKER.parse().unwrap(),
                };
                self.push_byte(syntax);
                self.read_amf0_string();
            }
            0x03 => {
                let syntax = SyntaxByte {
                    value: current_byte,
                    object_id: 4,
                    color: AMF0_OBJECT_MARKER.parse().unwrap(),
                };
                self.push_byte(syntax);
                self.read_amf0_object();
            }
            0x05 | 0x06 => {
                let syntax = SyntaxByte {
                    value: current_byte,
                    object_id: 6,
                    color: AMF0_NULL.parse().unwrap(),
                };
                self.push_byte(syntax);
            }
            // 0x07 => {}
            // 0x08 => {}
            // 0x09 => {}
            // 0x0A => {}
            // 0x0B => {}
            // 0x0C => {}
            // 0x0D => {}
            // 0x0F => {}
            0x10 => {
                let syntax = SyntaxByte {
                    value: current_byte,
                    object_id: 10,
                    color: AMF0_TYPED_OBJECT_MARKER.parse().unwrap(),
                };
                self.push_byte(syntax);
                self.read_amf0_typed_object();
            }
            0x11 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id: 1,
                    color: AMF0_SWITCH_MARKER.parse().unwrap(),
                });
                self.encoding = 3;
            }
            _ => self.out.push(SyntaxByte {
                value: current_byte,
                object_id: 0,
                color: "text-ctp-red".parse().unwrap(),
            }),
        }
    }
    pub fn read_amf3(&mut self) {
        let current_byte = self.read_byte();
        match current_byte {
            _ => self.out.push(SyntaxByte {
                value: current_byte,
                object_id: 1,
                color: "text-ctp-green".parse().unwrap(),
            }),
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        if self.read_head >= self.buffer.len() {
            return 0x20;
        }
        let b = self.buffer[self.read_head];
        self.read_head += 1;
        b
    }

    pub fn read_bytes(&mut self, len: usize) -> &[u8] {
        // if self.read_head + len >= self.buffer.len() {
        //     self.buffer[0] = 0x20;
        //     self.buffer[1] = 0x20;
        //     return &self.buffer[0..2];
        // }
        let b = &self.buffer[self.read_head..self.read_head + len];
        self.read_head += len;
        b
    }

    pub fn push_byte(&mut self, syntax_byte: SyntaxByte) {
        self.out.push(syntax_byte);
    }

    pub fn push_bytes(&mut self, syntax_byte: SyntaxByte, len: usize) {
        for i in 0..=len {
            let mut copy = syntax_byte.clone();
            let byte = self.read_byte();
            copy.value = byte;
            self.push_byte(copy);
        }
    }
}

#[derive(Clone)]
struct SyntaxByte {
    value: u8,
    object_id: usize,
    color: String,
}

#[component]
pub fn highlight_bytes(buffer: Vec<u8>, is_command: bool) -> Element {
    let mut reader = AMFReader::new(&buffer, is_command);
    reader.highlight();
    rsx! {
        for byte in reader.out {
            span {
                class: "{byte.color} hex",
                id: "{byte.object_id}",
                "{byte.value:02X} "
            }
        }
    }
}
