#[derive(Clone, Debug)]
pub enum ObjectProperties {
    Amf0StringProperties,
    Amf0ObjectProperties,
    Amf0TypedObjectProperties,
    AmfNoProperties,
}
