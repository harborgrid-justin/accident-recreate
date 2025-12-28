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
use quick_xml::{events::Event, Reader, Writer};
use serde_json::Value;
use std::io::Cursor;

pub struct XmlHandler;

#[async_trait]
impl ImportHandler for XmlHandler {
    async fn import(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Vec<DataRecord>> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let mut reader = Reader::from_reader(Cursor::new(data));
        reader.trim_text(true);

        let mut records = Vec::new();
        let mut current_record: Option<DataRecord> = None;
        let mut current_field = String::new();
        let mut current_value = String::new();
        let mut in_record = false;
        let mut record_count = 0;

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if name == "record" || name == "item" {
                        current_record = Some(DataRecord::new());
                        in_record = true;
                    } else if in_record {
                        current_field = name;
                        current_value.clear();
                    }
                }
                Ok(Event::Text(e)) => {
                    if in_record && !current_field.is_empty() {
                        current_value = e.unescape()?.to_string();
                    }
                }
                Ok(Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                    if name == "record" || name == "item" {
                        if let Some(record) = current_record.take() {
                            records.push(record);
                            record_count += 1;

                            if let Some(ref t) = tracker {
                                if t.is_cancelled().await {
                                    return Err(TransferError::Cancelled);
                                }
                                t.update(
                                    record_count,
                                    Some("Parsing XML records".to_string()),
                                )
                                .await;
                            }
                        }
                        in_record = false;
                    } else if in_record && !current_field.is_empty() {
                        if let Some(ref mut record) = current_record {
                            record.set(
                                current_field.clone(),
                                parse_xml_value(&current_value),
                            );
                        }
                        current_field.clear();
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(TransferError::Xml(format!("XML parse error: {}", e))),
                _ => {}
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
        let mut reader = Reader::from_reader(Cursor::new(data));
        reader.trim_text(true);

        // Try to parse the XML
        loop {
            match reader.read_event() {
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(TransferError::Validation(format!(
                        "Invalid XML: {}",
                        e
                    )))
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ExportHandler for XmlHandler {
    async fn export(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let mut writer = Writer::new(Cursor::new(Vec::new()));

        // Write XML declaration
        writer
            .write_event(Event::Decl(quick_xml::events::BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                None,
            )))
            .map_err(|e| TransferError::Xml(e.to_string()))?;

        // Write root element
        writer
            .write_event(Event::Start(
                quick_xml::events::BytesStart::new(&config.xml_root_element),
            ))
            .map_err(|e| TransferError::Xml(e.to_string()))?;

        // Write records
        for (idx, record) in records.iter().enumerate() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(idx as u64, Some("Writing XML records".to_string()))
                    .await;
            }

            // Start record element
            writer
                .write_event(Event::Start(quick_xml::events::BytesStart::new("record")))
                .map_err(|e| TransferError::Xml(e.to_string()))?;

            // Write fields
            for (key, value) in &record.fields {
                writer
                    .write_event(Event::Start(quick_xml::events::BytesStart::new(key)))
                    .map_err(|e| TransferError::Xml(e.to_string()))?;

                let value_str = format_xml_value(value);
                writer
                    .write_event(Event::Text(quick_xml::events::BytesText::new(
                        &value_str,
                    )))
                    .map_err(|e| TransferError::Xml(e.to_string()))?;

                writer
                    .write_event(Event::End(quick_xml::events::BytesEnd::new(key)))
                    .map_err(|e| TransferError::Xml(e.to_string()))?;
            }

            // End record element
            writer
                .write_event(Event::End(quick_xml::events::BytesEnd::new("record")))
                .map_err(|e| TransferError::Xml(e.to_string()))?;
        }

        // Close root element
        writer
            .write_event(Event::End(quick_xml::events::BytesEnd::new(
                &config.xml_root_element,
            )))
            .map_err(|e| TransferError::Xml(e.to_string()))?;

        let result = writer.into_inner().into_inner();

        if let Some(ref t) = tracker {
            t.complete().await;
        }

        Ok(Bytes::from(result))
    }

    async fn export_stream(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        self.export(records, config, tracker).await
    }
}

/// Parse XML text value to JSON value
fn parse_xml_value(value: &str) -> Value {
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
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }

    // Default to string
    Value::String(value.to_string())
}

/// Format JSON value for XML output
fn format_xml_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            serde_json::to_string(arr).unwrap_or_default()
        }
        Value::Object(obj) => {
            serde_json::to_string(obj).unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_xml_export_import() {
        let mut record = DataRecord::new();
        record.set("name".to_string(), Value::String("John Doe".to_string()));
        record.set("age".to_string(), Value::Number(30.into()));
        record.set("active".to_string(), Value::Bool(true));

        let records = vec![record];
        let config = TransferConfig::default();

        let handler = XmlHandler;
        let exported = handler.export(records.clone(), &config, None).await.unwrap();

        // Verify XML structure
        let xml_str = String::from_utf8(exported.to_vec()).unwrap();
        assert!(xml_str.contains("<record>"));
        assert!(xml_str.contains("<name>John Doe</name>"));

        let imported = handler.import(exported, &config, None).await.unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].get("name"), Some(&Value::String("John Doe".to_string())));
    }
}
