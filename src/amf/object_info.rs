use crate::amf::object_properties::TypeProperties;
use crate::amf::object_type::ObjectType;

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectInfo {
    pub(crate) object_id: usize,
    pub(crate) object_type: ObjectType,
    pub(crate) object_properties: TypeProperties,
}
