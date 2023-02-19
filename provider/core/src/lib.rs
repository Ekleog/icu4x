
pub struct AnyPayload;
struct AnyResponse {
    payload: Option<AnyPayload>,
}
impl TryFrom<DataResponse> for AnyResponse {
    type Error = DataError;
    fn try_from(other: DataResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            payload: other.payload.map(|p| p.try_unwrap_owned()).transpose()?,
        })
    }
}
enum DataErrorKind {
    MissingDataKey,
    MismatchedType(&'static [()]),
    Io(u8),
}
pub struct DataError {
    kind: DataErrorKind,
    key: Option<()>,
}
pub struct DataResponseMetadata;
pub struct DataPayload;
impl DataPayload {
    pub fn try_unwrap_owned(self) -> Result<AnyPayload, DataError> {
        loop {}
    }
}
pub struct DataResponse {
    pub payload: Option<DataPayload>,
}
pub use yoke;
