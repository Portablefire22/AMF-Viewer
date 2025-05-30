#[derive(Clone, Debug, PartialEq)]
pub enum TypeProperties {
    Amf0StringProperties,
    Amf0ObjectProperties,
    Amf0TypedObjectProperties,

    Amf3StringProperties(GenericProperties),
    Amf3ArrayProperties(GenericProperties),
    Amf3ObjectProperties(ObjectProperties),
    AmfNoProperties,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenericProperties {
    pub(crate) is_reference: bool,
    pub(crate) identifier: i32, // Reference ID when a reference, length when not
}

impl GenericProperties {
    pub fn new(is_reference: bool, identifier: i32) -> Self {
        Self {
            is_reference,
            identifier,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectProperties {
    pub(crate) is_reference: bool,
    pub(crate) property_count: usize,
    pub(crate) encoding: usize,
    pub(crate) externalisable: bool,
    pub(crate) dynamic: bool,
    pub(crate) object_type: String,
}

impl ObjectProperties {
    pub fn new(
        is_reference: bool,
        property_count: usize,
        encoding: usize,
        externalisable: bool,
        dynamic: bool,
        object_type: String,
    ) -> Self {
        Self {
            is_reference,
            property_count,
            encoding,
            externalisable,
            dynamic,
            object_type,
        }
    }
}
