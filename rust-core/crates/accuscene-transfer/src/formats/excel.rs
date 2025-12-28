use crate::{
    config::TransferConfig,
    error::{Result, TransferError},
    formats::{DataStream, ExportHandler, ImportHandler},
    progress::ProgressTracker,
    DataRecord,
};
use async_trait::async_trait;
use bytes::Bytes;
use calamine::{open_workbook_from_rs, DataType, Reader, Xlsx};
use futures::stream;
use serde_json::Value;
use std::io::Cursor;

pub struct ExcelHandler;

#[async_trait]
impl ImportHandler for ExcelHandler {
    async fn import(
        &self,
        data: Bytes,
        config: &TransferConfig,
        tracker: Option<ProgressTracker>,
    ) -> Result<Vec<DataRecord>> {
        if let Some(ref t) = tracker {
            t.start().await;
        }

        let cursor = Cursor::new(data.to_vec());
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
            .map_err(|e| TransferError::Excel(e.to_string()))?;

        // Get sheet
        let sheet_name = if let Some(ref name) = config.excel_sheet_name {
            name.clone()
        } else {
            workbook
                .sheet_names()
                .get(config.excel_sheet_index)
                .ok_or_else(|| {
                    TransferError::Excel(format!(
                        "Sheet index {} not found",
                        config.excel_sheet_index
                    ))
                })?
                .clone()
        };

        let range = workbook
            .worksheet_range(&sheet_name)
            .ok_or_else(|| TransferError::Excel(format!("Sheet '{}' not found", sheet_name)))?
            .map_err(|e| TransferError::Excel(e.to_string()))?;

        let mut records = Vec::new();
        let mut headers = Vec::new();
        let mut error_count = 0;

        for (row_idx, row) in range.rows().enumerate() {
            if let Some(ref t) = tracker {
                if t.is_cancelled().await {
                    return Err(TransferError::Cancelled);
                }
                t.update(row_idx as u64, Some("Reading Excel rows".to_string()))
                    .await;
            }

            // First row is headers
            if row_idx == 0 {
                headers = row
                    .iter()
                    .map(|cell| cell_to_string(cell))
                    .collect();
                continue;
            }

            // Parse data rows
            match parse_excel_row(row, &headers) {
                Ok(record) => records.push(record),
                Err(e) => {
                    error_count += 1;
                    if !config.continue_on_error || error_count >= config.error_threshold {
                        return Err(e);
                    }
                    tracing::warn!("Excel parse error at row {}: {}", row_idx, e);
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
        let records = self.import(data, config, tracker).await?;
        Ok(Box::pin(stream::iter(records.into_iter().map(Ok))))
    }

    async fn validate(&self, data: Bytes, config: &TransferConfig) -> Result<()> {
        let cursor = Cursor::new(data.to_vec());
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
            .map_err(|e| TransferError::Excel(e.to_string()))?;

        // Validate sheet exists
        if config.excel_sheet_index >= workbook.sheet_names().len() {
            return Err(TransferError::Validation(format!(
                "Sheet index {} out of range",
                config.excel_sheet_index
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl ExportHandler for ExcelHandler {
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

        // Note: For a complete implementation, you would use a library like rust_xlsxwriter
        // For now, we'll export as CSV-like format in a simple Excel structure
        // In production, use rust_xlsxwriter or similar for proper XLSX generation

        // This is a simplified implementation
        // A real implementation would use rust_xlsxwriter:
        // let mut workbook = Workbook::new();
        // let worksheet = workbook.add_worksheet();

        // For now, return error indicating full Excel export needs additional dependencies
        Err(TransferError::Excel(
            "Excel export requires rust_xlsxwriter. Use CSV export instead.".to_string()
        ))
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

/// Convert Excel cell to string
fn cell_to_string(cell: &DataType) -> String {
    match cell {
        DataType::Int(i) => i.to_string(),
        DataType::Float(f) => f.to_string(),
        DataType::String(s) => s.clone(),
        DataType::Bool(b) => b.to_string(),
        DataType::DateTime(dt) => dt.to_string(),
        DataType::Duration(d) => d.to_string(),
        DataType::Error(e) => format!("ERROR: {:?}", e),
        DataType::Empty => String::new(),
    }
}

/// Convert Excel cell to JSON value
fn cell_to_value(cell: &DataType) -> Value {
    match cell {
        DataType::Int(i) => Value::Number((*i).into()),
        DataType::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        DataType::String(s) => Value::String(s.clone()),
        DataType::Bool(b) => Value::Bool(*b),
        DataType::DateTime(dt) => Value::String(dt.to_string()),
        DataType::Duration(d) => Value::String(d.to_string()),
        DataType::Error(_) => Value::Null,
        DataType::Empty => Value::Null,
    }
}

/// Parse Excel row to DataRecord
fn parse_excel_row(row: &[DataType], headers: &[String]) -> Result<DataRecord> {
    let mut record = DataRecord::new();

    for (i, cell) in row.iter().enumerate() {
        if i < headers.len() {
            record.set(headers[i].clone(), cell_to_value(cell));
        }
    }

    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_conversions() {
        assert_eq!(cell_to_string(&DataType::Int(42)), "42");
        assert_eq!(cell_to_string(&DataType::String("test".to_string())), "test");
        assert_eq!(cell_to_string(&DataType::Bool(true)), "true");

        assert_eq!(cell_to_value(&DataType::Int(42)), Value::Number(42.into()));
        assert_eq!(cell_to_value(&DataType::Bool(true)), Value::Bool(true));
    }
}
