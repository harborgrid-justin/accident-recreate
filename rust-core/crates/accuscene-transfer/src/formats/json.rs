use crate::{
    config::TransferConfig,
    error::{Result, TransferError},
    formats::{DataStream, ExportHandler, ImportHandler},
    progress::ProgressTracker,
    DataRecord,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream;
use serde_json::{Value, Map};

pub struct JsonHandler;

#[async_trait]
impl ImportHandler for JsonHandler {
    async fn import(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Vec<DataRecord>> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let json: Value = serde_json::from_slice(&data)?;
        let mut records = Vec::new();
        let mut error_count = 0;

        match json {
            Value::Array(arr) => {
                for (idx, item) in arr.iter().enumerate() {
                    if let Some(ref t) = tracker {
                        if t.is_cancelled().await {
                            return Err(TransferError::Cancelled);
                        }
                        t.update(idx as u64, Some("Parsing JSON records".to_string()))
                            .await;
                    }

                    match parse_json_object(item) {
                        Ok(record) => records.push(record),
                        Err(e) => {
                            error_count += 1;
                            if !config.continue_on_error || error_count >= config.error_threshold {
                                return Err(e);
                            }
                            tracing::warn!("JSON parse error at index {}: {}", idx, e);
                        }
                    }
                }
            }
            Value::Object(_) => {
                // Single object
                records.push(parse_json_object(&json)?);
            }
            _ => {
                return Err(TransferError::Deserialization(
                    "JSON must be an object or array of objects".to_string(),
                ));
            }
        }

        if let Some(ref t) = tracker {
            t.complete().await;
        }

        Ok(records)
    }

    async fn import_stream(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<DataStream> {
        let records = self.import(data, config, tracker).await?;
        Ok(Box::pin(stream::iter(records.into_iter().map(Ok))))
    }

    async fn validate(&self, data: Bytes, _config: &TransferConfig) -> Result<()> {
        let json: Value = serde_json::from_slice(&data)?;

        match json {
            Value::Array(_) | Value::Object(_) => Ok(()),
            _ => Err(TransferError::Validation(
                "JSON must be an object or array".to_string(),
            )),
        }
    }
}

#[async_trait]
impl ExportHandler for JsonHandler {
    async fn export(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let mut json_records = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(idx as u64, Some("Converting to JSON".to_string()))
                    .await;
            }

            let mut obj = Map::new();
            for (key, value) in &record.fields {
                obj.insert(key.clone(), value.clone());
            }
            json_records.push(Value::Object(obj));
        }

        let output = Value::Array(json_records);

        let json_bytes = if config.json_pretty {
            serde_json::to_vec_pretty(&output)?
        } else {
            serde_json::to_vec(&output)?
        };

        if let Some(ref t) = tracker {
            t.complete().await;
        }

        Ok(Bytes::from(json_bytes))
    }

    async fn export_stream(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        // For JSON, streaming export is the same as regular export
        // In a real streaming implementation, you would use json-streamer crate
        self.export(records, config, tracker).await
    }
}

/// Parse JSON value to DataRecord
fn parse_json_object(value: &Value) -> Result<DataRecord> {
    match value {
        Value::Object(obj) => {
            let mut record = DataRecord::new();
            for (key, val) in obj {
                record.set(key.clone(), val.clone());
            }
            Ok(record)
        }
        _ => Err(TransferError::Deserialization(
            "Expected JSON object".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_json_export_import() {
        let mut record = DataRecord::new();
        record.set("name".to_string(), Value::String("John Doe".to_string()));
        record.set("age".to_string(), Value::Number(30.into()));
        record.set("active".to_string(), Value::Bool(true));

        let records = vec![record];
        let config = TransferConfig::default();

        let handler = JsonHandler;
        let exported = handler.export(records.clone(), &config, None).await.unwrap();
        let imported = handler.import(exported, &config, None).await.unwrap();

        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].get("name"), Some(&Value::String("John Doe".to_string())));
        assert_eq!(imported[0].get("age"), Some(&Value::Number(30.into())));
        assert_eq!(imported[0].get("active"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_json_pretty_print() {
        let mut record = DataRecord::new();
        record.set("test".to_string(), Value::String("value".to_string()));

        let records = vec![record];
        let config = TransferConfig::default().with_json_pretty(true);

        let handler = JsonHandler;
        let exported = handler.export(records, &config, None).await.unwrap();
        let json_str = String::from_utf8(exported.to_vec()).unwrap();

        assert!(json_str.contains('\n')); // Pretty printed has newlines
    }
}
