#[derive(Clone, Debug)]
pub enum ObjectProperties {
    Amf0StringProperties,
    Amf0ObjectProperties,
    Amf0TypedObjectProperties,

    Amf3StringProperties(GenericProperties),
    Amf3ArrayProperties(GenericProperties),
    AmfNoProperties,
}

#[derive(Clone, Debug)]
pub struct GenericProperties {
    is_reference: bool,
    identifier: i32, // Reference ID when a reference, length when not
}

impl GenericProperties {
    pub fn new(is_reference: bool, identifier: i32) -> Self {
        Self {
            is_reference,
            identifier,
        }
    }
}
