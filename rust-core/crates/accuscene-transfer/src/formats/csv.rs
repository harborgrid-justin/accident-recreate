use crate::{
    config::TransferConfig,
    error::{Result, TransferError},
    formats::{DataStream, ExportHandler, ImportHandler},
    progress::ProgressTracker,
    DataRecord,
};
use async_trait::async_trait;
use bytes::Bytes;
use csv::{Reader, Writer};
use futures::stream;
use serde_json::Value;
use std::io::Cursor;

pub struct CsvHandler;

#[async_trait]
impl ImportHandler for CsvHandler {
    async fn import(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Vec<DataRecord>> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let cursor = Cursor::new(data);
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(config.csv_delimiter as u8)
            .has_headers(config.csv_has_header)
            .from_reader(cursor);

        let headers = if config.csv_has_header {
            reader.headers()?.iter().map(|h| h.to_string()).collect()
        } else {
            // Generate default headers
            let first_record = reader.records().next();
            if let Some(Ok(record)) = first_record {
                (0..record.len())
                    .map(|i| format!("column_{}", i + 1))
                    .collect()
            } else {
                vec![]
            }
        };

        let mut records = Vec::new();
        let mut error_count = 0;

        for (idx, result) in reader.records().enumerate() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(idx as u64, Some("Reading CSV records".to_string())).await;
            }

            match result {
                Ok(csv_record) => {
                    let mut record = DataRecord::new();
                    for (i, field) in csv_record.iter().enumerate() {
                        if i < headers.len() {
                            record.set(
                                headers[i].clone(),
                                parse_csv_value(field),
                            );
                        }
                    }
                    records.push(record);
                }
                Err(e) => {
                    error_count += 1;
                    if !config.continue_on_error || error_count >= config.error_threshold {
                        return Err(TransferError::Csv(e));
                    }
                    tracing::warn!("CSV parse error at row {}: {}", idx, e);
                }
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
        // For CSV, we'll read in chunks
        let records = self.import(data, config, tracker).await?;
        Ok(Box::pin(stream::iter(records.into_iter().map(Ok))))
    }

    async fn validate(&self, data: Bytes, config: &TransferConfig) -> Result<()> {
        let cursor = Cursor::new(data);
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(config.csv_delimiter as u8)
            .has_headers(config.csv_has_header)
            .from_reader(cursor);

        // Validate headers
        if config.csv_has_header {
            reader.headers()?;
        }

        // Validate at least one record exists
        if reader.records().next().is_none() {
            return Err(TransferError::Validation("CSV file is empty".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl ExportHandler for CsvHandler {
    async fn export(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        if records.is_empty() {
            return Ok(Bytes::new());
        }

        let mut wtr = Writer::from_writer(vec![]);

        // Write headers
        let headers: Vec<String> = records[0].field_names();
        wtr.write_record(&headers)?;

        // Write records
        for (idx, record) in records.iter().enumerate() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(idx as u64, Some("Writing CSV records".to_string())).await;
            }

            let row: Vec<String> = headers
                .iter()
                .map(|h| {
                    record
                        .get(h)
                        .map(|v| format_csv_value(v))
                        .unwrap_or_default()
                })
                .collect();
            wtr.write_record(&row)?;
        }

        wtr.flush()?;
        let data = wtr.into_inner().map_err(|e| {
            TransferError::Serialization(format!("CSV write error: {}", e))
        })?;

        if let Some(ref t) = tracker {
            t.complete().await;
        }

        Ok(Bytes::from(data))
    }

    async fn export_stream(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        // For CSV, streaming export is the same as regular export
        self.export(records, config, tracker).await
    }
}

/// Parse CSV value to JSON value
fn parse_csv_value(value: &str) -> Value {
    // Try to parse as number
    if let Ok(num) = value.parse::<i64>() {
        return Value::Number(num.into());
    }
    if let Ok(num) = value.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(num) {
            return Value::Number(num);
        }
    }

    // Try to parse as boolean
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" => return Value::Bool(true),
        "false" | "no" | "0" => return Value::Bool(false),
        _ => {}
    }

    // Default to string
    Value::String(value.to_string())
}

/// Format JSON value for CSV output
fn format_csv_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            // Join array elements
            arr.iter()
                .map(|v| format_csv_value(v))
                .collect::<Vec<_>>()
                .join("; ")
        }
        Value::Object(_) => {
            // Serialize objects as JSON
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_csv_export_import() {
        let mut record = DataRecord::new();
        record.set("name".to_string(), Value::String("John Doe".to_string()));
        record.set("age".to_string(), Value::Number(30.into()));
        record.set("active".to_string(), Value::Bool(true));

        let records = vec![record];
        let config = TransferConfig::default();

        let handler = CsvHandler;
        let exported = handler.export(records.clone(), &config, None).await.unwrap();
        let imported = handler.import(exported, &config, None).await.unwrap();

        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].get("name"), Some(&Value::String("John Doe".to_string())));
    }
}
