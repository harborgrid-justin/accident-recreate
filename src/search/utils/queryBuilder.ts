/**
 * Client-side query builder utilities
 */

import type {
  SearchQuery,
  SearchFilters,
  SearchRequest,
  SearchOptions,
  DateRange,
} from '../types';

export class QueryBuilder {
  private query: SearchQuery;
  private filters: SearchFilters;
  private options: SearchOptions;

  constructor() {
    this.query = { text: '' };
    this.filters = {};
    this.options = {};
  }

  /**
   * Set the query text
   */
  text(text: string): this {
    this.query.text = text;
    return this;
  }

  /**
   * Set search fields
   */
  fields(fields: string[]): this {
    this.query.fields = fields;
    return this;
  }

  /**
   * Enable/disable fuzzy matching
   */
  fuzzy(enabled: boolean = true): this {
    this.query.fuzzy = enabled;
    return this;
  }

  /**
   * Set boost factor
   */
  boost(factor: number): this {
    this.query.boost = factor;
    return this;
  }

  /**
   * Set query operator
   */
  operator(op: 'AND' | 'OR'): this {
    this.query.operator = op;
    return this;
  }

  /**
   * Add category filter
   */
  category(category: string | string[]): this {
    this.filters.categories = Array.isArray(category) ? category : [category];
    return this;
  }

  /**
   * Add status filter
   */
  status(status: string | string[]): this {
    this.filters.statuses = Array.isArray(status) ? status : [status];
    return this;
  }

  /**
   * Add severity filter
   */
  severity(severity: string | string[]): this {
    this.filters.severities = Array.isArray(severity) ? severity : [severity];
    return this;
  }

  /**
   * Add tag filter
   */
  tags(tags: string[]): this {
    this.filters.tags = tags;
    return this;
  }

  /**
   * Add created by filter
   */
  createdBy(users: string[]): this {
    this.filters.createdBy = users;
    return this;
  }

  /**
   * Set date range filter
   */
  dateRange(start: Date, end: Date): this {
    this.filters.dateRange = { start, end };
    return this;
  }

  /**
   * Set date range for last N days
   */
  lastNDays(days: number): this {
    const end = new Date();
    const start = new Date();
    start.setDate(start.getDate() - days);
    this.filters.dateRange = { start, end };
    return this;
  }

  /**
   * Add custom filter
   */
  custom(key: string, value: any): this {
    if (!this.filters.custom) {
      this.filters.custom = {};
    }
    this.filters.custom[key] = value;
    return this;
  }

  /**
   * Set pagination
   */
  page(page: number, perPage: number = 20): this {
    this.options.page = page;
    this.options.perPage = perPage;
    return this;
  }

  /**
   * Set sorting
   */
  sort(field: string, order: 'asc' | 'desc' = 'desc'): this {
    this.options.sortBy = field;
    this.options.sortOrder = order;
    return this;
  }

  /**
   * Enable/disable highlighting
   */
  highlight(enabled: boolean = true): this {
    this.options.highlight = enabled;
    return this;
  }

  /**
   * Request specific facets
   */
  facets(fields: string[]): this {
    this.options.facets = fields;
    return this;
  }

  /**
   * Build the search request
   */
  build(): SearchRequest {
    return {
      query: this.query,
      filters: Object.keys(this.filters).length > 0 ? this.filters : undefined,
      options: Object.keys(this.options).length > 0 ? this.options : undefined,
    };
  }

  /**
   * Reset builder to initial state
   */
  reset(): this {
    this.query = { text: '' };
    this.filters = {};
    this.options = {};
    return this;
  }
}

/**
 * Parse a Lucene-style query string
 */
export function parseLuceneQuery(queryString: string): SearchQuery {
  const query: SearchQuery = {
    text: '',
    fields: [],
    fuzzy: false,
    operator: 'OR',
  };

  const tokens = queryString.split(/\s+/);
  const textParts: string[] = [];

  for (const token of tokens) {
    if (token.includes(':')) {
      // Field-specific query: field:value
      const [field, value] = token.split(':', 2);
      if (field && value) {
        query.fields = query.fields || [];
        query.fields.push(field);
        textParts.push(value);
      }
    } else if (token.endsWith('~')) {
      // Fuzzy query indicator
      query.fuzzy = true;
      textParts.push(token.slice(0, -1));
    } else if (token.toUpperCase() === 'AND') {
      query.operator = 'AND';
    } else if (token.toUpperCase() === 'OR') {
      query.operator = 'OR';
    } else {
      textParts.push(token);
    }
  }

  query.text = textParts.join(' ');
  return query;
}

/**
 * Build a simple text query
 */
export function simpleQuery(text: string): SearchRequest {
  return new QueryBuilder().text(text).build();
}

/**
 * Build a query from URL search params
 */
export function fromURLParams(params: URLSearchParams): SearchRequest {
  const builder = new QueryBuilder();

  const q = params.get('q');
  if (q) builder.text(q);

  const fields = params.get('fields');
  if (fields) builder.fields(fields.split(','));

  const fuzzy = params.get('fuzzy');
  if (fuzzy === 'true') builder.fuzzy(true);

  const operator = params.get('operator') as 'AND' | 'OR' | null;
  if (operator) builder.operator(operator);

  // Filters
  const categories = params.get('categories');
  if (categories) builder.category(categories.split(','));

  const statuses = params.get('statuses');
  if (statuses) builder.status(statuses.split(','));

  const severities = params.get('severities');
  if (severities) builder.severity(severities.split(','));

  const tags = params.get('tags');
  if (tags) builder.tags(tags.split(','));

  // Date range
  const startDate = params.get('startDate');
  const endDate = params.get('endDate');
  if (startDate && endDate) {
    builder.dateRange(new Date(startDate), new Date(endDate));
  }

  // Options
  const page = params.get('page');
  const perPage = params.get('perPage');
  if (page) {
    builder.page(parseInt(page), perPage ? parseInt(perPage) : 20);
  }

  const sortBy = params.get('sortBy');
  const sortOrder = params.get('sortOrder') as 'asc' | 'desc' | null;
  if (sortBy) {
    builder.sort(sortBy, sortOrder || 'desc');
  }

  return builder.build();
}

/**
 * Convert search request to URL params
 */
export function toURLParams(request: SearchRequest): URLSearchParams {
  const params = new URLSearchParams();

  if (request.query.text) {
    params.set('q', request.query.text);
  }

  if (request.query.fields && request.query.fields.length > 0) {
    params.set('fields', request.query.fields.join(','));
  }

  if (request.query.fuzzy) {
    params.set('fuzzy', 'true');
  }

  if (request.query.operator) {
    params.set('operator', request.query.operator);
  }

  // Filters
  if (request.filters) {
    if (request.filters.categories) {
      params.set('categories', request.filters.categories.join(','));
    }

    if (request.filters.statuses) {
      params.set('statuses', request.filters.statuses.join(','));
    }

    if (request.filters.severities) {
      params.set('severities', request.filters.severities.join(','));
    }

    if (request.filters.tags) {
      params.set('tags', request.filters.tags.join(','));
    }

    if (request.filters.dateRange) {
      params.set('startDate', request.filters.dateRange.start.toISOString());
      params.set('endDate', request.filters.dateRange.end.toISOString());
    }
  }

  // Options
  if (request.options) {
    if (request.options.page !== undefined) {
      params.set('page', request.options.page.toString());
    }

    if (request.options.perPage) {
      params.set('perPage', request.options.perPage.toString());
    }

    if (request.options.sortBy) {
      params.set('sortBy', request.options.sortBy);
    }

    if (request.options.sortOrder) {
      params.set('sortOrder', request.options.sortOrder);
    }
  }

  return params;
}
