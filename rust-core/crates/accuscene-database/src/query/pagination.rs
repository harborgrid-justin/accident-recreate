//! Pagination utilities for database queries

use serde::{Deserialize, Serialize};

/// Offset-based pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number (1-indexed)
    pub page: usize,
    /// Number of items per page
    pub page_size: usize,
}

impl Pagination {
    /// Create a new pagination with default page size
    pub fn new(page: usize) -> Self {
        Self {
            page: page.max(1),
            page_size: 20,
        }
    }

    /// Create pagination with custom page size
    pub fn with_page_size(page: usize, page_size: usize) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.max(1).min(100), // Cap at 100 items
        }
    }

    /// Get the offset for SQL queries
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.page_size
    }

    /// Get the limit for SQL queries
    pub fn limit(&self) -> usize {
        self.page_size
    }

    /// Calculate total pages from total items
    pub fn total_pages(&self, total_items: usize) -> usize {
        (total_items + self.page_size - 1) / self.page_size
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(1)
    }
}

/// Pagination result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResult<T> {
    /// Current page data
    pub data: Vec<T>,
    /// Current page number
    pub page: usize,
    /// Items per page
    pub page_size: usize,
    /// Total number of items
    pub total_items: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
}

impl<T> PaginationResult<T> {
    /// Create a new pagination result
    pub fn new(data: Vec<T>, pagination: &Pagination, total_items: usize) -> Self {
        let total_pages = pagination.total_pages(total_items);

        Self {
            data,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
            total_pages,
            has_next: pagination.page < total_pages,
            has_prev: pagination.page > 1,
        }
    }

    /// Get the next page number
    pub fn next_page(&self) -> Option<usize> {
        if self.has_next {
            Some(self.page + 1)
        } else {
            None
        }
    }

    /// Get the previous page number
    pub fn prev_page(&self) -> Option<usize> {
        if self.has_prev {
            Some(self.page - 1)
        } else {
            None
        }
    }
}

/// Cursor-based pagination for efficient large dataset pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPagination {
    /// Cursor for the next page (usually an ID or timestamp)
    pub cursor: Option<String>,
    /// Number of items per page
    pub page_size: usize,
}

impl CursorPagination {
    /// Create a new cursor pagination
    pub fn new(cursor: Option<String>, page_size: usize) -> Self {
        Self {
            cursor,
            page_size: page_size.max(1).min(100),
        }
    }

    /// Create cursor pagination with default page size
    pub fn with_cursor(cursor: Option<String>) -> Self {
        Self::new(cursor, 20)
    }
}

impl Default for CursorPagination {
    fn default() -> Self {
        Self::new(None, 20)
    }
}

/// Cursor pagination result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPaginationResult<T> {
    /// Current page data
    pub data: Vec<T>,
    /// Cursor for the next page
    pub next_cursor: Option<String>,
    /// Whether there are more items
    pub has_more: bool,
    /// Page size
    pub page_size: usize,
}

impl<T> CursorPaginationResult<T> {
    /// Create a new cursor pagination result
    pub fn new(data: Vec<T>, next_cursor: Option<String>, page_size: usize) -> Self {
        let has_more = next_cursor.is_some();

        Self {
            data,
            next_cursor,
            has_more,
            page_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.page_size, 20);
        assert_eq!(pagination.offset(), 20);
        assert_eq!(pagination.limit(), 20);
    }

    #[test]
    fn test_pagination_total_pages() {
        let pagination = Pagination::with_page_size(1, 10);
        assert_eq!(pagination.total_pages(95), 10);
        assert_eq!(pagination.total_pages(100), 10);
        assert_eq!(pagination.total_pages(101), 11);
    }

    #[test]
    fn test_pagination_result() {
        let pagination = Pagination::with_page_size(2, 10);
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = PaginationResult::new(data, &pagination, 25);

        assert_eq!(result.page, 2);
        assert_eq!(result.total_items, 25);
        assert_eq!(result.total_pages, 3);
        assert!(result.has_next);
        assert!(result.has_prev);
        assert_eq!(result.next_page(), Some(3));
        assert_eq!(result.prev_page(), Some(1));
    }

    #[test]
    fn test_cursor_pagination() {
        let cursor_page = CursorPagination::with_cursor(Some("abc123".to_string()));
        assert_eq!(cursor_page.cursor, Some("abc123".to_string()));
        assert_eq!(cursor_page.page_size, 20);
    }
}
