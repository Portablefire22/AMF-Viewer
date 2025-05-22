use crate::amf::object_properties::ObjectProperties;
use crate::amf::object_type::ObjectType;

#[derive(Clone, Debug)]
pub struct ObjectInfo {
    pub(crate) object_id: usize,
    pub(crate) object_type: ObjectType,
    pub(crate) object_properties: ObjectProperties,
}
