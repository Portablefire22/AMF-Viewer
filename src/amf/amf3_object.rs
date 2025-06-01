use std::collections::HashMap;

pub struct AmfObject {
    pub(crate) property_count: usize,
    pub(crate) encoding: i32,
    pub(crate) externalisable: bool,
    pub(crate) dynamic: bool,
    pub(crate) object_type: String,
    pub(crate) properties: HashMap<String, Option<isize>>,
}

impl AmfObject {
    pub fn new(
        encoding: i32,
        externalisable: bool,
        dynamic: bool,
        object_type: String,
        properties: HashMap<String, Option<isize>>,
    ) -> Self {
        Self {
            property_count: properties.keys().len(),
            encoding,
            externalisable,
            dynamic,
            object_type,
            properties,
        }
    }
}
