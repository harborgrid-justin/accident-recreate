//! Table widget implementation
//!
//! Displays data in tabular format with sorting, filtering, and pagination

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

use super::{Widget, WidgetConfig, WidgetData, InteractionEvent, ExportFormat};
use crate::error::{WidgetError, WidgetResult};

/// Column data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    /// String/text
    String,
    /// Numeric value
    Number,
    /// Boolean
    Boolean,
    /// Date
    Date,
    /// DateTime
    DateTime,
    /// Currency
    Currency,
    /// Percentage
    Percentage,
    /// Custom
    Custom,
}

/// Column alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnAlign {
    /// Left aligned
    Left,
    /// Center aligned
    Center,
    /// Right aligned
    Right,
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    /// Column unique identifier
    pub id: String,

    /// Column header label
    pub label: String,

    /// Data type
    pub data_type: ColumnType,

    /// Field name in data
    pub field: String,

    /// Alignment
    pub align: ColumnAlign,

    /// Width in pixels (None for auto)
    pub width: Option<u32>,

    /// Minimum width in pixels
    pub min_width: Option<u32>,

    /// Sortable
    pub sortable: bool,

    /// Filterable
    pub filterable: bool,

    /// Visible
    pub visible: bool,

    /// Frozen (sticky column)
    pub frozen: bool,

    /// Custom renderer function name
    pub renderer: Option<String>,

    /// Format string for dates/numbers
    pub format: Option<String>,
}

impl ColumnDef {
    /// Create a new column definition
    pub fn new(id: String, label: String, field: String, data_type: ColumnType) -> Self {
        let align = match data_type {
            ColumnType::Number | ColumnType::Currency | ColumnType::Percentage => ColumnAlign::Right,
            ColumnType::Boolean => ColumnAlign::Center,
            _ => ColumnAlign::Left,
        };

        Self {
            id,
            label,
            data_type,
            field,
            align,
            width: None,
            min_width: Some(100),
            sortable: true,
            filterable: true,
            visible: true,
            frozen: false,
            renderer: None,
            format: None,
        }
    }

    /// Set width
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set frozen
    pub fn frozen(mut self) -> Self {
        self.frozen = true;
        self
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

/// Sort configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortConfig {
    /// Column to sort by
    pub column_id: String,

    /// Sort direction
    pub direction: SortDirection,
}

/// Filter operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    /// Equal to
    Equals,
    /// Not equal to
    NotEquals,
    /// Contains (for strings)
    Contains,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// In list
    In,
    /// Not in list
    NotIn,
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Column to filter
    pub column_id: String,

    /// Filter operator
    pub operator: FilterOperator,

    /// Filter value
    pub value: JsonValue,
}

/// Pagination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Current page (0-indexed)
    pub page: usize,

    /// Page size
    pub page_size: usize,

    /// Total rows
    pub total_rows: usize,

    /// Total pages
    pub total_pages: usize,
}

impl PaginationConfig {
    /// Create new pagination config
    pub fn new(page_size: usize, total_rows: usize) -> Self {
        let total_pages = (total_rows + page_size - 1) / page_size;
        Self {
            page: 0,
            page_size,
            total_rows,
            total_pages,
        }
    }

    /// Go to next page
    pub fn next_page(&mut self) -> bool {
        if self.page + 1 < self.total_pages {
            self.page += 1;
            true
        } else {
            false
        }
    }

    /// Go to previous page
    pub fn prev_page(&mut self) -> bool {
        if self.page > 0 {
            self.page -= 1;
            true
        } else {
            false
        }
    }

    /// Get start index for current page
    pub fn start_index(&self) -> usize {
        self.page * self.page_size
    }

    /// Get end index for current page
    pub fn end_index(&self) -> usize {
        ((self.page + 1) * self.page_size).min(self.total_rows)
    }
}

/// Table row data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// Row unique identifier
    pub id: String,

    /// Row data (column_id -> value)
    pub data: HashMap<String, JsonValue>,

    /// Row metadata
    pub metadata: Option<JsonValue>,

    /// Row is selected
    pub selected: bool,
}

impl TableRow {
    /// Create a new table row
    pub fn new(id: String, data: HashMap<String, JsonValue>) -> Self {
        Self {
            id,
            data,
            metadata: None,
            selected: false,
        }
    }

    /// Get cell value
    pub fn get_cell(&self, column_id: &str) -> Option<&JsonValue> {
        self.data.get(column_id)
    }
}

/// Table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    /// Column definitions
    pub columns: Vec<ColumnDef>,

    /// Enable sorting
    pub sorting_enabled: bool,

    /// Enable filtering
    pub filtering_enabled: bool,

    /// Enable pagination
    pub pagination_enabled: bool,

    /// Default page size
    pub default_page_size: usize,

    /// Enable row selection
    pub selection_enabled: bool,

    /// Multi-row selection
    pub multi_select: bool,

    /// Enable column reordering
    pub column_reorder_enabled: bool,

    /// Enable column resizing
    pub column_resize_enabled: bool,

    /// Enable row virtualization (for large datasets)
    pub virtualization_enabled: bool,

    /// Show row numbers
    pub show_row_numbers: bool,

    /// Striped rows
    pub striped: bool,

    /// Dense table (compact spacing)
    pub dense: bool,
}

impl TableConfig {
    /// Create a new table configuration
    pub fn new(columns: Vec<ColumnDef>) -> Self {
        Self {
            columns,
            sorting_enabled: true,
            filtering_enabled: true,
            pagination_enabled: true,
            default_page_size: 25,
            selection_enabled: true,
            multi_select: true,
            column_reorder_enabled: true,
            column_resize_enabled: true,
            virtualization_enabled: false,
            show_row_numbers: false,
            striped: true,
            dense: false,
        }
    }

    /// Get column by ID
    pub fn get_column(&self, column_id: &str) -> Option<&ColumnDef> {
        self.columns.iter().find(|c| c.id == column_id)
    }

    /// Get visible columns
    pub fn visible_columns(&self) -> Vec<&ColumnDef> {
        self.columns.iter().filter(|c| c.visible).collect()
    }
}

/// Table widget implementation
pub struct TableWidget {
    config: WidgetConfig,
    data: Option<WidgetData>,
    table_config: TableConfig,
    rows: Vec<TableRow>,
    sort: Option<SortConfig>,
    filters: Vec<FilterConfig>,
    pagination: Option<PaginationConfig>,
}

impl TableWidget {
    /// Create a new table widget
    pub fn new(config: WidgetConfig, table_config: TableConfig) -> Self {
        let pagination = if table_config.pagination_enabled {
            Some(PaginationConfig::new(table_config.default_page_size, 0))
        } else {
            None
        };

        Self {
            config,
            data: None,
            table_config,
            rows: Vec::new(),
            sort: None,
            filters: Vec::new(),
            pagination,
        }
    }

    /// Update rows
    pub fn update_rows(&mut self, rows: Vec<TableRow>) {
        self.rows = rows;
        if let Some(ref mut pagination) = self.pagination {
            pagination.total_rows = self.rows.len();
            pagination.total_pages = (pagination.total_rows + pagination.page_size - 1) / pagination.page_size;
        }
    }

    /// Apply sort
    pub fn apply_sort(&mut self, column_id: String, direction: SortDirection) {
        self.sort = Some(SortConfig { column_id, direction });
        // In a real implementation, this would actually sort the rows
    }

    /// Add filter
    pub fn add_filter(&mut self, filter: FilterConfig) {
        self.filters.push(filter);
    }

    /// Clear filters
    pub fn clear_filters(&mut self) {
        self.filters.clear();
    }

    /// Get selected rows
    pub fn get_selected_rows(&self) -> Vec<&TableRow> {
        self.rows.iter().filter(|r| r.selected).collect()
    }

    /// Get current page rows
    fn get_page_rows(&self) -> Vec<&TableRow> {
        if let Some(ref pagination) = self.pagination {
            let start = pagination.start_index();
            let end = pagination.end_index();
            self.rows.iter().skip(start).take(end - start).collect()
        } else {
            self.rows.iter().collect()
        }
    }

    /// Serialize table data
    fn serialize_data(&self) -> JsonValue {
        let page_rows = self.get_page_rows();

        json!({
            "columns": self.table_config.columns.iter().map(|c| json!({
                "id": c.id,
                "label": c.label,
                "field": c.field,
                "data_type": match c.data_type {
                    ColumnType::String => "string",
                    ColumnType::Number => "number",
                    ColumnType::Boolean => "boolean",
                    ColumnType::Date => "date",
                    ColumnType::DateTime => "datetime",
                    ColumnType::Currency => "currency",
                    ColumnType::Percentage => "percentage",
                    ColumnType::Custom => "custom",
                },
                "align": match c.align {
                    ColumnAlign::Left => "left",
                    ColumnAlign::Center => "center",
                    ColumnAlign::Right => "right",
                },
                "width": c.width,
                "min_width": c.min_width,
                "sortable": c.sortable,
                "filterable": c.filterable,
                "visible": c.visible,
                "frozen": c.frozen,
                "renderer": c.renderer,
                "format": c.format,
            })).collect::<Vec<_>>(),
            "rows": page_rows.iter().map(|r| json!({
                "id": r.id,
                "data": r.data,
                "metadata": r.metadata,
                "selected": r.selected,
            })).collect::<Vec<_>>(),
            "sort": self.sort.as_ref().map(|s| json!({
                "column_id": s.column_id,
                "direction": match s.direction {
                    SortDirection::Asc => "asc",
                    SortDirection::Desc => "desc",
                },
            })),
            "filters": self.filters,
            "pagination": self.pagination.as_ref().map(|p| json!({
                "page": p.page,
                "page_size": p.page_size,
                "total_rows": p.total_rows,
                "total_pages": p.total_pages,
            })),
            "config": {
                "sorting_enabled": self.table_config.sorting_enabled,
                "filtering_enabled": self.table_config.filtering_enabled,
                "pagination_enabled": self.table_config.pagination_enabled,
                "selection_enabled": self.table_config.selection_enabled,
                "multi_select": self.table_config.multi_select,
                "striped": self.table_config.striped,
                "dense": self.table_config.dense,
                "show_row_numbers": self.table_config.show_row_numbers,
            },
        })
    }
}

#[async_trait]
impl Widget for TableWidget {
    fn config(&self) -> &WidgetConfig {
        &self.config
    }

    fn data(&self) -> Option<&WidgetData> {
        self.data.as_ref()
    }

    async fn fetch_data(&mut self) -> WidgetResult<WidgetData> {
        let table_data = self.serialize_data();
        let data = WidgetData::new(table_data);
        self.data = Some(data.clone());
        Ok(data)
    }

    fn validate(&self) -> WidgetResult<()> {
        if self.table_config.columns.is_empty() {
            return Err(WidgetError::invalid_config("At least one column is required"));
        }

        for column in &self.table_config.columns {
            if column.id.is_empty() {
                return Err(WidgetError::invalid_config("Column ID cannot be empty"));
            }
            if column.field.is_empty() {
                return Err(WidgetError::invalid_config("Column field cannot be empty"));
            }
        }

        Ok(())
    }

    async fn handle_interaction(&mut self, event: InteractionEvent) -> WidgetResult<()> {
        match event.event_type.as_str() {
            "sort" => {
                let column_id = event.data["column_id"]
                    .as_str()
                    .ok_or_else(|| WidgetError::data_error("Missing column_id"))?
                    .to_string();

                let direction = match event.data["direction"].as_str() {
                    Some("desc") => SortDirection::Desc,
                    _ => SortDirection::Asc,
                };

                self.apply_sort(column_id, direction);
                Ok(())
            }
            "page_change" => {
                if let Some(ref mut pagination) = self.pagination {
                    if let Some(page) = event.data["page"].as_u64() {
                        pagination.page = page as usize;
                    }
                }
                Ok(())
            }
            "select_row" => {
                let row_id = event.data["row_id"]
                    .as_str()
                    .ok_or_else(|| WidgetError::data_error("Missing row_id"))?;

                if let Some(row) = self.rows.iter_mut().find(|r| r.id == row_id) {
                    row.selected = !row.selected;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn export(&self, format: ExportFormat) -> WidgetResult<Vec<u8>> {
        match format {
            ExportFormat::Json => {
                let json_data = self.serialize_data();
                serde_json::to_vec_pretty(&json_data)
                    .map_err(|e| WidgetError::render_error(format!("JSON export failed: {}", e)))
            }
            ExportFormat::Csv => {
                let visible_columns = self.table_config.visible_columns();
                let mut csv = String::new();

                // Header row
                let headers: Vec<&str> = visible_columns.iter().map(|c| c.label.as_str()).collect();
                csv.push_str(&headers.join(","));
                csv.push('\n');

                // Data rows
                for row in &self.rows {
                    let values: Vec<String> = visible_columns
                        .iter()
                        .map(|col| {
                            row.get_cell(&col.id)
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string()
                        })
                        .collect();
                    csv.push_str(&values.join(","));
                    csv.push('\n');
                }

                Ok(csv.into_bytes())
            }
            _ => Err(WidgetError::render_error(format!("Export format {:?} not supported", format))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_def() {
        let col = ColumnDef::new(
            "id".to_string(),
            "ID".to_string(),
            "id".to_string(),
            ColumnType::Number,
        );

        assert_eq!(col.align, ColumnAlign::Right);
        assert!(col.sortable);
    }

    #[test]
    fn test_pagination() {
        let mut pagination = PaginationConfig::new(10, 25);
        assert_eq!(pagination.total_pages, 3);
        assert_eq!(pagination.start_index(), 0);
        assert_eq!(pagination.end_index(), 10);

        pagination.next_page();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.start_index(), 10);
    }

    #[test]
    fn test_table_row() {
        let mut data = HashMap::new();
        data.insert("id".to_string(), json!(1));
        data.insert("name".to_string(), json!("Test"));

        let row = TableRow::new("row-1".to_string(), data);
        assert_eq!(row.get_cell("id"), Some(&json!(1)));
        assert_eq!(row.get_cell("name"), Some(&json!("Test")));
    }
}
