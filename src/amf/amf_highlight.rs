use crate::amf::object_info::ObjectInfo;
use crate::amf::object_properties::ObjectProperties::{
    Amf0ObjectProperties, Amf0StringProperties, Amf0TypedObjectProperties, AmfNoProperties,
};
use crate::amf::object_type::ObjectType;
use crate::amf::object_type::ObjectType::Amf0Number;
use crate::amf::syntax_byte::SyntaxByte;
use dioxus::html::completions::CompleteWithBraces::object;
use dioxus::logger::tracing;
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
const AMF0_OBJECT_MARKER: &'static str = "text-ctp-teal/80";
const AMF0_NULL: &'static str = "text-ctp-rosewater";
const AMF0_TYPED_OBJECT: &'static str = "text-ctp-yellow";
const AMF0_TYPED_OBJECT_MARKER: &'static str = "text-ctp-yellow/80";
const AMF0_SWITCH_MARKER: &'static str = "text-ctp-pink/80";

// I fucking LOVE Action Message Format
pub struct AMFReader {
    buffer: Vec<u8>,
    read_head: usize,
    pub(crate) out: Vec<SyntaxByte>,
    encoding: u8,
    current_layer: u8, // Change colour depending on object layer
    pub(crate) objects: Vec<ObjectInfo>,
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
                objects: Vec::new(),
            }
        } else {
            AMFReader {
                buffer: buffer.clone(),
                read_head: 0,
                out: Vec::new(),
                encoding: 0,
                current_layer: 0,
                objects: Vec::new(),
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

    fn read_amf0_integer(&mut self, object_id: Option<usize>) {
        let object_id: usize = match object_id {
            Some(id) => id,
            None => self.objects.len(),
        };
        let syntax = SyntaxByte {
            value: 0,
            object_id,
            color: AMF0_NUMBER.parse().unwrap(),
        };
        let bytes = self.push_bytes(syntax, 7);

        let number = f64::from_be_bytes(<[u8; 8]>::try_from(bytes).unwrap());

        self.objects.push(ObjectInfo {
            object_id,
            object_type: Amf0Number(number),
            object_properties: AmfNoProperties,
        });
    }

    fn read_amf0_bool(&mut self, object_id: Option<usize>) {
        let object_id: usize = match object_id {
            Some(id) => id,
            None => self.objects.len(),
        };
        let byte = self.read_byte();
        let mut syntax = SyntaxByte {
            value: byte,
            object_id,
            color: AMF0_BOOL_TRUE.parse().unwrap(),
        };
        if byte == 0 {
            syntax.color = AMF0_BOOL_FALSE.parse().unwrap();
        }
        self.push_byte(syntax);

        let info = ObjectInfo {
            object_id,
            object_type: ObjectType::Amf0Bool(byte == 1),
            object_properties: AmfNoProperties,
        };

        self.objects.push(info);
    }

    fn read_amf0_string(&mut self, object_id: Option<usize>) -> String {
        self.read_amf0_utf8(None, object_id)
    }

    fn read_amf0_utf_length(&mut self, colour: Option<String>, object_id: usize) -> u16 {
        let colour = colour.unwrap_or_else(|| AMF0_STRING.parse().unwrap());
        // I love fighting the borrow checker
        let len_bytes = &self.buffer[self.read_head..self.read_head + 2];
        self.read_head += 2;
        let length = ((len_bytes[0] as u16) << 8) | len_bytes[1] as u16;
        for i in len_bytes {
            let syntax = SyntaxByte {
                value: *i,
                object_id,
                color: colour.clone(),
            };
            self.out.push(syntax);
        }
        length
    }

    fn read_amf0_utf8(&mut self, colour: Option<String>, object_id: Option<usize>) -> String {
        let object_id: usize = match object_id {
            Some(id) => id,
            None => self.objects.len(),
        };

        let length = self.read_amf0_utf_length(colour.clone(), object_id);

        let colour = colour.unwrap_or_else(|| AMF0_STRING.parse().unwrap());
        let mut out = String::new();
        self.read_bytes(length as usize)
            .read_to_string(&mut out)
            .unwrap();
        for i in out.as_bytes() {
            let syntax = SyntaxByte {
                value: *i,
                object_id,
                color: colour.clone(),
            };
            self.push_byte(syntax);
        }

        let info = ObjectInfo {
            object_id,
            object_type: ObjectType::Amf0String(out.clone()),
            object_properties: Amf0StringProperties,
        };

        self.objects.push(info);

        out
    }

    fn read_amf0_object(&mut self, object_id: Option<usize>) {
        let object_id: usize = match object_id {
            Some(id) => id,
            None => self.objects.len(),
        };

        let info = ObjectInfo {
            object_id,
            object_type: ObjectType::Amf0Object,
            object_properties: Amf0ObjectProperties,
        };

        self.objects.push(info);

        self.current_layer += 1;
        loop {
            let key = self.read_amf0_utf8(
                Some(format!("{}/{}", AMF0_OBJECT, 100 - self.current_layer * 20)),
                None,
            );
            tracing::debug!("Key: '{}'", key);
            if key.is_empty() {
                let b = self.read_byte();
                let syntax = SyntaxByte {
                    value: b,
                    object_id,
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

    fn read_amf0_typed_object(&mut self, object_id: Option<usize>) {
        let object_id: usize = match object_id {
            Some(id) => id,
            None => self.objects.len(),
        };

        let info = ObjectInfo {
            object_id,
            object_type: ObjectType::Amf0TypedObject,
            object_properties: Amf0TypedObjectProperties,
        };

        self.objects.push(info);

        self.current_layer += 1;
        self.read_amf0_utf8(
            Some(format!(
                "{}/{}",
                AMF0_TYPED_OBJECT,
                100 - self.current_layer * 20
            )),
            None,
        );
        self.read_amf0_object(Some(object_id));
    }

    pub fn read_amf0(&mut self) {
        let current_byte = self.read_byte();
        let object_id = self.objects.len();
        match current_byte {
            0x00 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF0_NUMBER_MARKER.parse().unwrap(),
                });
                self.read_amf0_integer(Some(object_id));
            }
            0x01 => {
                let mut syntax = SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF0_BOOL_TRUE_MARKER.parse().unwrap(),
                };
                if current_byte == 0 {
                    syntax.color = AMF0_BOOL_FALSE_MARKER.parse().unwrap();
                }
                self.push_byte(syntax);
                self.read_amf0_bool(Some(object_id));
            }
            0x02 => {
                let syntax = SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF0_STRING_MARKER.parse().unwrap(),
                };
                self.push_byte(syntax);
                self.read_amf0_string(Some(object_id));
            }
            0x03 => {
                let syntax = SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF0_OBJECT_MARKER.parse().unwrap(),
                };
                self.push_byte(syntax);
                self.read_amf0_object(Some(object_id));
            }
            0x05 | 0x06 => {
                let syntax = SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF0_NULL.parse().unwrap(),
                };
                let info = ObjectInfo {
                    object_id,
                    object_type: ObjectType::Amf0Null,
                    object_properties: AmfNoProperties,
                };

                self.objects.push(info);
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
                    object_id,
                    color: AMF0_TYPED_OBJECT_MARKER.parse().unwrap(),
                };
                self.push_byte(syntax);
                self.read_amf0_typed_object(Some(object_id));
            }
            0x11 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF0_SWITCH_MARKER.parse().unwrap(),
                });
                self.encoding = 3;
                let info = ObjectInfo {
                    object_id,
                    object_type: ObjectType::Amf0Switch,
                    object_properties: AmfNoProperties,
                };

                self.objects.push(info);
            }
            _ => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: "text-ctp-red".parse().unwrap(),
                });
                let info = ObjectInfo {
                    object_id,
                    object_type: ObjectType::Amf0Undefined,
                    object_properties: AmfNoProperties,
                };

                self.objects.push(info);
            }
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

    pub fn push_bytes(&mut self, syntax_byte: SyntaxByte, len: usize) -> Vec<u8> {
        let mut arr = Vec::new();
        for _ in 0..=len {
            let mut copy = syntax_byte.clone();
            let byte = self.read_byte();
            copy.value = byte;
            arr.push(byte);
            self.push_byte(copy);
        }
        arr
    }
}
