use crate::amf::amf3_object::AmfObject;
use crate::amf::object_info::ObjectInfo;
use crate::amf::object_properties::TypeProperties::{
    Amf0ObjectProperties, Amf0StringProperties, Amf0TypedObjectProperties, Amf3ArrayProperties,
    Amf3ObjectProperties, Amf3StringProperties, AmfNoProperties,
};
use crate::amf::object_properties::{GenericProperties, ObjectProperties};
use crate::amf::object_type::ObjectType;
use crate::amf::object_type::ObjectType::{
    Amf0Number, Amf0Undefined, Amf3Array, Amf3Object, Amf3String, Amf3Undefined,
};
use crate::amf::syntax_byte::SyntaxByte;
use dioxus::logger::tracing;
use std::collections::HashMap;
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
const AMF0_TYPED_OBJECT: &'static str = "text-ctp-mauve";
const AMF0_TYPED_OBJECT_MARKER: &'static str = "text-ctp-yellow/80";
const AMF0_SWITCH_MARKER: &'static str = "text-ctp-pink/80";
const AMF3_UNDEFINED: &'static str = "text-ctp-pink";
const AMF3_NULL: &'static str = "text-ctp-rosewater";
const AMF3_FALSE: &'static str = "text-ctp-red";
const AMF3_TRUE: &'static str = "text-ctp-green";
const AMF3_INTEGER: &'static str = "text-ctp-sky";
const AMF3_DOUBLE: &'static str = "text-ctp-blue";
const AMF3_STRING: &'static str = "text-ctp-yellow";
const AMF3_DATE: &'static str = "text-ctp-rosewater";
const AMF3_ARRAY: &'static str = "text-ctp-lavender";
const AMF3_OBJECT: &'static str = "text-ctp-mauve";
const AMF_ERROR: &'static str = "text-red-500";

// I fucking LOVE Action Message Format
pub struct AMFReader {
    buffer: Vec<u8>,
    read_head: usize,
    pub(crate) out: Vec<SyntaxByte>,
    encoding: u8,
    current_layer: u8, // Change colour depending on object layer
    pub(crate) objects: HashMap<isize, ObjectInfo>,
    strings: Vec<String>,

    is_error: bool, // Are we in an error state right now?
}

impl AMFReader {
    pub fn new(buffer: &Vec<u8>, is_command: bool) -> Self {
        if is_command {
            //let encoding = buffer[0];
            AMFReader {
                buffer: buffer.clone(),
                read_head: 1,
                out: Vec::new(),
                encoding: 0,
                current_layer: 0,
                objects: HashMap::new(),
                strings: Vec::new(),
                is_error: false,
            }
        } else {
            AMFReader {
                buffer: buffer.clone(),
                read_head: 0,
                out: Vec::new(),
                encoding: 0,
                current_layer: 0,
                objects: HashMap::new(),
                strings: Vec::new(),
                is_error: false,
            }
        }
    }

    pub fn highlight(&mut self) {
        while &self.read_head < &self.buffer.len() {
            if self.is_error {
                let b = match self.read_byte() {
                    None => break,
                    Some(b) => *b,
                };

                let syn = SyntaxByte {
                    value: b,
                    object_id: -1,
                    color: AMF_ERROR.parse().unwrap(),
                };
                self.push_byte(syn)
            }
            if self.encoding == 0 {
                self.read_amf0();
            } else {
                self.read_amf3();
            }
        }
        if self.is_error {
            self.objects.insert(
                -1,
                ObjectInfo {
                    object_id: -1,
                    object_type: Amf0Undefined,
                    object_properties: AmfNoProperties,
                },
            );
        }
    }

    fn read_amf0_integer(&mut self, object_id: Option<isize>) {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };
        let syntax = SyntaxByte {
            value: 0,
            object_id,
            color: AMF0_NUMBER.parse().unwrap(),
        };
        let bytes = self.push_bytes(syntax, 7);

        let number = f64::from_be_bytes(<[u8; 8]>::try_from(bytes).unwrap());

        self.objects.insert(
            object_id,
            ObjectInfo {
                object_id,
                object_type: Amf0Number(number),
                object_properties: AmfNoProperties,
            },
        );
    }

    fn read_amf0_bool(&mut self, object_id: Option<isize>) {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };
        let byte = match self.read_byte() {
            Some(b) => *b,
            None => {
                self.is_error = true;
                return;
            }
        };
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

        self.objects.insert(object_id, info);
    }

    fn read_amf0_string(&mut self, object_id: Option<isize>) -> String {
        self.read_amf0_utf8(None, object_id)
    }

    fn read_amf0_utf_length(&mut self, colour: Option<String>, object_id: isize) -> u16 {
        let colour = colour.unwrap_or_else(|| AMF0_STRING.parse().unwrap());
        // I love fighting the borrow checker
        let len_bytes = (&self.buffer).get(self.read_head..self.read_head + 2);
        let len_bytes = match len_bytes {
            Some(b) => b,
            None => {
                self.is_error = true;
                return 0;
            }
        };
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

    fn read_amf0_utf8(&mut self, colour: Option<String>, object_id: Option<isize>) -> String {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };

        let length = self.read_amf0_utf_length(colour.clone(), object_id);

        let colour = colour.unwrap_or_else(|| AMF0_STRING.parse().unwrap());
        let mut out = String::new();
        match self.read_bytes(length as usize) {
            Some(mut b) => match b.read_to_string(&mut out) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Could not read stream: {:?}", e);
                    self.is_error = true;
                    return String::new();
                }
            },
            None => {
                self.is_error = true;
                return String::new();
            }
        };
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

        self.objects.insert(object_id, info);

        out
    }

    fn read_amf0_object(&mut self, object_id: Option<isize>) {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };

        let info = ObjectInfo {
            object_id,
            object_type: ObjectType::Amf0Object,
            object_properties: Amf0ObjectProperties,
        };

        self.objects.insert(object_id, info);

        self.current_layer += 1;
        loop {
            let key = self.read_amf0_utf8(
                Some(format!("{}/{}", AMF0_OBJECT, 100 - self.current_layer * 20)),
                None,
            );
            tracing::debug!("Key: '{}'", key);
            if key.is_empty() {
                let b = match self.read_byte() {
                    Some(b) => *b,
                    None => {
                        self.is_error = true;
                        return;
                    }
                };
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

    fn read_amf0_typed_object(&mut self, object_id: Option<isize>) {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };

        let info = ObjectInfo {
            object_id,
            object_type: ObjectType::Amf0TypedObject,
            object_properties: Amf0TypedObjectProperties,
        };

        self.objects.insert(object_id, info);

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

    pub fn read_amf0(&mut self) -> isize {
        if self.is_error {
            return -1;
        }
        let current_byte = match self.read_byte() {
            Some(byte) => *byte,
            None => {
                self.is_error = true;
                return -1;
            }
        };
        let object_id = self.objects.len() as isize;
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

                self.objects.insert(object_id, info);
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

                self.objects.insert(object_id, info);
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

                self.objects.insert(object_id, info);
            }
        }
        object_id
    }

    pub fn read_amf3_integer(&mut self, object_id: Option<isize>) -> i32 {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };
        let syntax = SyntaxByte {
            value: 0,
            object_id,
            color: AMF3_INTEGER.parse().unwrap(),
        };
        let out = self.amf3_integer(syntax);
        let info = ObjectInfo {
            object_id,
            object_type: ObjectType::Amf3Integer(out),
            object_properties: AmfNoProperties,
        };
        self.objects.insert(object_id, info);
        out
    }

    fn amf3_integer(&mut self, mut syntax: SyntaxByte) -> i32 {
        let mut out: i32 = 0;
        let mut i = 0;
        let mut current_byte = match self.read_byte() {
            Some(b) => *b,
            None => {
                self.is_error = true;
                return -1;
            }
        };
        syntax.value = current_byte;
        self.out.push(syntax.clone());

        while current_byte & 0x80 != 0 && i < 3 {
            out = (out << 7) + (current_byte & 0x7F) as i32;
            current_byte = match self.read_byte() {
                Some(b) => *b,
                None => {
                    self.is_error = true;
                    return -1;
                }
            };
            let mut syntax = syntax.clone();
            syntax.value = current_byte;
            self.out.push(syntax);
            i += 1;
        }
        if i < 3 {
            out = (out << 7) | current_byte as i32;
        } else {
            out = (out << 8) | current_byte as i32;
        }
        out
    }

    pub fn read_amf3_string_length(&mut self, object_id: Option<isize>) -> i32 {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };
        let syntax = SyntaxByte {
            value: 0,
            object_id,
            color: AMF3_INTEGER.parse().unwrap(),
        };
        // self.out.push(syntax.clone());
        self.amf3_integer(syntax)
    }

    pub fn read_amf3_string(&mut self, object_id: Option<isize>) -> String {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };

        let mut refe = self.read_amf3_string_length(Some(object_id));
        let inline = (refe & 0x01) == 1;
        refe >>= 1;
        if inline {
            if refe == 0 {
                return String::from("");
            }
            let out = self.read_amf3_utf8(refe, Some(object_id));
            let info = ObjectInfo {
                object_id,
                object_type: ObjectType::Amf3String(out.clone()),
                object_properties: Amf3StringProperties(GenericProperties::new(
                    false,
                    out.len() as i32,
                )),
            };
            self.objects.insert(object_id, info);
            self.strings.push(out.clone());
            out
        } else {
            let s: String = match self.strings.get(refe as usize) {
                Some(s) => s.clone(),
                None => String::from("String Not Found"),
            };

            let info = ObjectInfo {
                object_id,
                object_type: ObjectType::Amf3String(s.clone()),
                object_properties: Amf3StringProperties(GenericProperties::new(true, refe)),
            };
            self.objects.insert(object_id, info);
            s
        }
    }

    pub fn read_amf3_utf8(&mut self, length: i32, object_id: Option<isize>) -> String {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };
        let syntax = SyntaxByte {
            value: 0,
            object_id,
            color: AMF3_STRING.parse().unwrap(),
        };
        let string_bytes = self.push_bytes(syntax, length as usize - 1);
        let out = String::from_utf8(string_bytes.clone()).unwrap_or(format!("{:?}", string_bytes));

        out
    }

    pub fn read_amf3_array(&mut self, object_id: Option<isize>) -> Vec<isize> {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };

        let mut out = Vec::new();
        let mut refer = self.read_amf3_string_length(Some(object_id));
        refer >>= 1;

        for _ in 0..refer {
            let id = self.read_amf3();
            out.push(id);
        }
        let obj = ObjectInfo {
            object_id,
            object_type: Amf3Array(out.clone()),
            object_properties: Amf3ArrayProperties(GenericProperties::new(false, refer)),
        };
        self.objects.insert(object_id, obj.clone());
        out
    }

    fn read_amf3_object(&mut self, object_id: Option<isize>) -> AmfObject {
        let object_id: isize = match object_id {
            Some(id) => id,
            None => self.objects.len() as isize,
        };

        let mut result = AmfObject::new(0, false, false, String::new(), HashMap::new());

        let mut refer = self.read_amf3_string_length(Some(object_id));
        refer >>= 1;
        let is_reference = refer & 0x01 == 0;
        if !is_reference {
            let is_inline_class_def = (refer & 0x01) != 0;
            refer >>= 1;
            if is_inline_class_def {
                result.encoding = refer & 0x03;
                result.externalisable = (result.encoding & 0x01) != 0;
                result.dynamic = ((refer >> 1) & 0x01) != 0;
                result.property_count = refer as usize >> 2;
                result.object_type = self.read_amf3_string(None);

                for i in 0..result.property_count {
                    let key = self.read_amf3_string(None);
                    result.properties.insert(key, None);
                }
            } else {
            }
            if result.externalisable {
            } else {
                for (_, (key, value)) in result.properties.iter_mut().enumerate() {
                    *value = Some(self.read_amf3());
                }
                if result.dynamic {
                    loop {
                        if self.read_head == self.buffer.len() {
                            break;
                        }
                        let byte = self.buffer[self.read_head];
                        if byte == 1 {
                            self.read_head += 1;
                            break;
                        }
                        let key = self.read_amf3_string(None);
                        result.properties.insert(key, Some(self.read_amf3()));
                    }
                }
            }
        } else {
        }

        let handle = &result;
        let obj = ObjectInfo {
            object_id,
            object_type: Amf3Object(handle.properties.clone()),
            object_properties: Amf3ObjectProperties(ObjectProperties::new(
                is_reference,
                handle.properties.len(),
                handle.encoding as usize,
                handle.externalisable,
                handle.dynamic,
                handle.object_type.clone(),
            )),
        };

        self.objects.insert(object_id, obj);

        result
    }

    pub fn read_amf3(&mut self) -> isize {
        if self.is_error {
            return -1;
        }
        let object_id = self.objects.len() as isize;
        let current_byte = match self.read_byte() {
            Some(byte) => *byte,
            None => {
                self.is_error = true;
                return -1;
            }
        };
        match current_byte {
            0x01 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF3_NULL.parse().unwrap(),
                });
                let info = ObjectInfo {
                    object_id,
                    object_type: ObjectType::Amf3Null,
                    object_properties: AmfNoProperties,
                };
                self.objects.insert(object_id, info);
            }
            0x02 => {
                // False
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF3_FALSE.parse().unwrap(),
                });
                let info = ObjectInfo {
                    object_id,
                    object_type: ObjectType::Amf3False,
                    object_properties: AmfNoProperties,
                };
                self.objects.insert(object_id, info);
            }
            0x03 => {
                // True
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF3_TRUE.parse().unwrap(),
                });
                let info = ObjectInfo {
                    object_id,
                    object_type: ObjectType::Amf3True,
                    object_properties: AmfNoProperties,
                };
                self.objects.insert(object_id, info);
            }
            0x04 => {
                // Integer
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF3_INTEGER.parse().unwrap(),
                });
                self.read_amf3_integer(Some(object_id));
            }
            0x05 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF0_NUMBER_MARKER.parse().unwrap(),
                });
                self.read_amf0_integer(Some(object_id));
            }
            0x06 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF3_STRING.parse().unwrap(),
                });
                self.read_amf3_string(Some(object_id));
            }
            0x08 => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF3_ARRAY.parse().unwrap(),
                });
                self.read_amf3_array(Some(object_id));
            }
            0x0A => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: AMF3_OBJECT.parse().unwrap(),
                });
                self.read_amf3_object(Some(object_id));
            }
            _ => {
                self.out.push(SyntaxByte {
                    value: current_byte,
                    object_id,
                    color: "text-ctp-maroon".parse().unwrap(),
                });
                let info = ObjectInfo {
                    object_id,
                    object_type: ObjectType::Amf3Undefined,
                    object_properties: AmfNoProperties,
                };
                self.objects.insert(object_id, info);
            }
        }
        object_id
    }

    pub fn read_byte(&mut self) -> Option<&u8> {
        let b = self.buffer.get(self.read_head);
        if b.is_some() {
            self.read_head += 1;
        }
        b
    }

    pub fn read_bytes(&mut self, len: usize) -> Option<&[u8]> {
        let b = (&self.buffer).get(self.read_head..self.read_head + len);
        if b.is_some() {
            self.read_head += len;
        }
        b
    }

    pub fn push_byte(&mut self, syntax_byte: SyntaxByte) {
        self.out.push(syntax_byte);
    }

    pub fn push_bytes(&mut self, syntax_byte: SyntaxByte, len: usize) -> Vec<u8> {
        let mut arr = Vec::new();
        for _ in 0..=len {
            let mut copy = syntax_byte.clone();
            let byte = match self.read_byte() {
                Some(byte) => *byte,
                None => {
                    self.is_error = true;
                    break;
                }
            };
            copy.value = byte;
            arr.push(byte);
            self.push_byte(copy);
        }
        arr
    }
}
