/// Retains the runtime checks required before untrusted remote data enters typed code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemotePayloadShape {
    Number,
    String,
    Boolean,
    Array(Box<Self>),
    Record(Vec<RemotePayloadField>),
}

/// Couples a record field name to the runtime shape required for its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemotePayloadField {
    name: String,
    shape: RemotePayloadShape,
}

impl RemotePayloadField {
    pub(crate) fn from_parts(parts: (String, RemotePayloadShape)) -> Self {
        Self {
            name: parts.0,
            shape: parts.1,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn shape(&self) -> &RemotePayloadShape {
        &self.shape
    }
}
