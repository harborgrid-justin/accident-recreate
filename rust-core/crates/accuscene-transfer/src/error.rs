use thiserror::Error;

/// Transfer operation errors
#[derive(Error, Debug)]
pub enum TransferError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Excel error: {0}")]
    Excel(String),

    #[error("XML error: {0}")]
    Xml(String),

    #[error("PDF error: {0}")]
    Pdf(String),

    #[error("Archive error: {0}")]
    Archive(#[from] zip::result::ZipError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("Field mapping error: {0}")]
    FieldMapping(String),

    #[error("Transform error: {0}")]
    Transform(String),

    #[error("Format not supported: {0}")]
    UnsupportedFormat(String),

    #[error("File too large: {0} bytes (max: {1} bytes)")]
    FileTooLarge(u64, u64),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Progress tracking error: {0}")]
    Progress(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type for transfer operations
pub type Result<T> = std::result::Result<T, TransferError>;

impl From<serde_json::Error> for TransferError {
    fn from(err: serde_json::Error) -> Self {
        TransferError::Serialization(err.to_string())
    }
}

impl From<quick_xml::Error> for TransferError {
    fn from(err: quick_xml::Error) -> Self {
        TransferError::Xml(err.to_string())
    }
}

impl From<quick_xml::DeError> for TransferError {
    fn from(err: quick_xml::DeError) -> Self {
        TransferError::Xml(err.to_string())
    }
}
