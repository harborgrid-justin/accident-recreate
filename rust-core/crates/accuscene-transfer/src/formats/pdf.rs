use crate::{
    config::TransferConfig,
    error::{Result, TransferError},
    formats::ExportHandler,
    progress::ProgressTracker,
    DataRecord,
};
use async_trait::async_trait;
use bytes::Bytes;
use printpdf::*;
use serde_json::Value;

pub struct PdfHandler;

#[async_trait]
impl ExportHandler for PdfHandler {
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

        // Create PDF document
        let (doc, page1, layer1) = PdfDocument::new(
            "AccuScene Export",
            Mm(210.0), // A4 width
            Mm(297.0), // A4 height
            "Layer 1",
        );

        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Get headers from first record
        let headers: Vec<String> = records[0].field_names();

        // Define layout
        let font_size = 10.0;
        let line_height = 5.0;
        let margin_left = 10.0;
        let margin_top = 280.0;
        let col_width = 40.0;

        // Load font (built-in)
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| TransferError::Pdf(format!("Font error: {:?}", e)))?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| TransferError::Pdf(format!("Font error: {:?}", e)))?;

        let mut y_pos = margin_top;

        // Write title
        current_layer.use_text(
            "AccuScene Data Export",
            16.0,
            Mm(margin_left),
            Mm(y_pos),
            &font_bold,
        );
        y_pos -= line_height * 2.0;

        // Write headers
        for (idx, header) in headers.iter().enumerate() {
            let x_pos = margin_left + (idx as f32 * col_width);
            current_layer.use_text(
                header,
                font_size,
                Mm(x_pos),
                Mm(y_pos),
                &font_bold,
            );
        }
        y_pos -= line_height;

        // Write records
        let mut current_page = page1;
        let mut current_layer_ref = current_layer;

        for (record_idx, record) in records.iter().enumerate() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(record_idx as u64, Some("Generating PDF".to_string()))
                    .await;
            }

            // Check if we need a new page
            if y_pos < 20.0 {
                let (new_page, new_layer) = doc.add_page(
                    Mm(210.0),
                    Mm(297.0),
                    "Layer 1",
                );
                current_page = new_page;
                current_layer_ref = doc.get_page(current_page).get_layer(new_layer);
                y_pos = margin_top;
            }

            // Write record fields
            for (idx, header) in headers.iter().enumerate() {
                let x_pos = margin_left + (idx as f32 * col_width);
                let value = record
                    .get(header)
                    .map(|v| format_pdf_value(v))
                    .unwrap_or_default();

                // Truncate long values
                let display_value = if value.len() > 15 {
                    format!("{}...", &value[..12])
                } else {
                    value
                };

                current_layer_ref.use_text(
                    &display_value,
                    font_size,
                    Mm(x_pos),
                    Mm(y_pos),
                    &font,
                );
            }

            y_pos -= line_height;
        }

        // Add metadata
        doc.metadata.document_title = Some("AccuScene Export".to_string());
        doc.metadata.creation_date = Some(chrono::Utc::now());
        doc.metadata.creator = Some("AccuScene Enterprise".to_string());

        // Save to bytes
        let mut buffer = Vec::new();
        doc.save(&mut buffer)
            .map_err(|e| TransferError::Pdf(format!("PDF save error: {:?}", e)))?;

        if let Some(ref t) = tracker {
            t.complete().await;
        }

        Ok(Bytes::from(buffer))
    }

    async fn export_stream(
        &self,
        records: Vec<DataRecord>,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Bytes> {
        // PDF generation is not easily streamable
        self.export(records, config, tracker).await
    }
}

/// Format JSON value for PDF output
fn format_pdf_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Object(obj) => format!("{{{} fields}}", obj.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pdf_export() {
        let mut record = DataRecord::new();
        record.set("name".to_string(), Value::String("John Doe".to_string()));
        record.set("age".to_string(), Value::Number(30.into()));

        let records = vec![record];
        let config = TransferConfig::default();

        let handler = PdfHandler;
        let result = handler.export(records, &config, None).await;

        assert!(result.is_ok());
        let pdf_data = result.unwrap();
        assert!(!pdf_data.is_empty());

        // Verify PDF magic number
        assert_eq!(&pdf_data[0..4], b"%PDF");
    }
}
