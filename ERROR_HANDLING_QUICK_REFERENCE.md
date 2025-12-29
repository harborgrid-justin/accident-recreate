# Error Handling Quick Reference Guide

## Quick Start

### Rust - Basic Usage

```rust
use accuscene_errors::{AccuSceneError, Result, error, ensure};

// Create an error
let err = AccuSceneError::validation("Invalid input");

// Use in function return
fn process() -> Result<()> {
    ensure!(data.is_valid(), validation, "Data must be valid");
    Ok(())
}

// Add context
fn load_data() -> Result<Data> {
    read_file("data.json")
        .context("Failed to load data file")?
}

// Use macros
let err = error!(database, "Connection failed", details: "Timeout after 30s");
```

### TypeScript - Basic Usage

```typescript
import { AccuSceneError, useErrorHandler } from '@/errors';

// Create an error
const err = AccuSceneError.validation('Invalid input');

// In React component
function MyComponent() {
  const { handleError } = useErrorHandler();

  const doSomething = async () => {
    try {
      await fetchData();
    } catch (err) {
      handleError(err, 'Failed to fetch data');
    }
  };
}

// Wrap with error boundary
<ErrorBoundary>
  <MyComponent />
</ErrorBoundary>
```

## Common Patterns

### Rust

#### Retry Pattern
```rust
use accuscene_errors::{RetryExecutor, RetryPolicy};

let executor = RetryExecutor::new(RetryPolicy::aggressive());
let result = executor.execute(|| async {
    fetch_data().await
}).await?;
```

#### Circuit Breaker
```rust
use accuscene_errors::CircuitBreaker;

let breaker = CircuitBreaker::new(5, 2, Duration::from_secs(60));
let result = breaker.execute(|| async {
    call_service().await
}).await?;
```

#### Error with Metadata
```rust
let err = AccuSceneError::network("Request failed")
    .with_metadata("url", "https://api.example.com")
    .with_metadata("method", "GET")
    .with_location(file!(), line!());
```

### TypeScript

#### Async Error Handling
```typescript
import { useAsyncError } from '@/errors';

function DataLoader() {
  const { execute, isLoading, error } = useAsyncError();

  const load = () => execute(
    async () => await fetch('/api/data'),
    { context: 'Loading data' }
  );
}
```

#### Retry with Timeout
```typescript
import { retry, withTimeout } from '@/errors';

const result = await retry(
  () => withTimeout(fetch('/api/data'), 5000),
  { maxAttempts: 3 }
);
```

#### Error Recovery
```typescript
import { useErrorRecovery } from '@/errors';

function ResilientComponent() {
  const { error } = useErrorHandler();
  const { canRecover, executeRecovery } = useErrorRecovery(error);

  return canRecover && (
    <button onClick={executeRecovery}>Retry</button>
  );
}
```

## Error Code Cheat Sheet

### Client Errors
- `VALIDATION` - Invalid input (400)
- `AUTHENTICATION` - Auth required (401)
- `AUTHORIZATION` - No permission (403)
- `NOT_FOUND` - Resource not found (404)
- `CONFLICT` - Resource conflict (409)
- `RATE_LIMIT` - Too many requests (429)

### Server Errors
- `INTERNAL` - Internal error (500)
- `DATABASE` - Database error (500)
- `NETWORK` - Network error (500)
- `TIMEOUT` - Timeout (504)
- `UNAVAILABLE` - Service down (503)

### Domain Errors
- `PHYSICS` - Physics simulation
- `RENDERING` - Rendering issue
- `CACHE` - Cache problem
- `ANALYTICS` - Analytics error
- ... and more

## Factory Methods

### Rust
```rust
AccuSceneError::validation("msg")
AccuSceneError::authentication("msg")
AccuSceneError::authorization("msg")
AccuSceneError::not_found("msg")
AccuSceneError::conflict("msg")
AccuSceneError::internal("msg")
AccuSceneError::database("msg")
AccuSceneError::network("msg")
AccuSceneError::timeout("msg")
AccuSceneError::rate_limit("msg")
```

### TypeScript
```typescript
AccuSceneError.validation('msg')
AccuSceneError.authentication('msg')
AccuSceneError.authorization('msg')
AccuSceneError.notFound('msg')
AccuSceneError.conflict('msg')
AccuSceneError.internal('msg')
AccuSceneError.database('msg')
AccuSceneError.network('msg')
AccuSceneError.timeout('msg')
AccuSceneError.rateLimit('msg')
```

## Macros (Rust Only)

```rust
// Create error with location
error!(validation, "Invalid input")

// Ensure condition
ensure!(x > 0, validation, "X must be positive")

// Log and return error
log_error!(internal, "Operation failed")

// Bail early
bail!(not_found, "Resource not found")

// Wrap error with context
wrap_err!(operation(), "Context message")

// Try with context
try_with_context!(operation(), "Context message")

// Create recoverable error
recoverable_error!(network, "Temporary failure")

// Create fatal error
fatal_error!(internal, "Critical failure")
```

## Hooks (TypeScript Only)

```typescript
// Basic error handling
const { error, handleError, clearError } = useErrorHandler();

// Async operations
const { execute, isLoading, error } = useAsyncError();

// Retry logic
const { execute, attempt, isRetrying } = useRetry(asyncFn, options);

// Error recovery
const { recoveryAction, executeRecovery } = useErrorRecovery(error);

// Error boundary
const { error, throwError, reset } = useErrorBoundary();

// Error metrics
const { metrics, recentErrors } = useErrorMetrics();
```

## Utilities (TypeScript Only)

```typescript
// Wrap function with error handling
const safeFunc = withErrorHandling(myFunc, 'Context');

// Ensure condition
ensure(x > 0, ErrorCode.VALIDATION, 'X must be positive');

// Ensure valid
ensureValid(isValid, 'Must be valid');

// Ensure found
ensureFound(user, 'User not found');

// Retry operation
await retry(asyncFn, { maxAttempts: 3 });

// Add timeout
await withTimeout(promise, 5000);

// Catch error
const result = await catchError(promise, 'context');

// Aggregate errors
const aggregated = aggregateErrors([err1, err2]);

// Try-catch wrapper
const result = tryCatch(() => operation());
const result = await tryCatchAsync(() => asyncOp());

// Unwrap result
const value = unwrap(result); // throws if error
const value = unwrapOr(result, defaultValue);
```

## Error Severity Levels

```
INFO < WARNING < LOW < MEDIUM < HIGH < CRITICAL
```

- **CRITICAL**: Database, Internal, Resource exhausted
- **HIGH**: Auth, External service, Cluster, Crypto
- **MEDIUM**: Network, Timeout, File system, Domain errors
- **LOW**: Validation, Not found, Conflict, Rate limit
- **WARNING**: Deprecated, Unimplemented

## Recovery Actions

- **RETRY**: Automatically retry the operation
- **FALLBACK**: Use alternative data or method
- **SKIP**: Continue without the failed operation
- **ABORT**: Stop the entire process
- **USER_INTERVENTION**: Ask user to fix the issue
- **USE_CACHE**: Fall back to cached data
- **DEGRADE**: Reduce service functionality

## Best Practices

### DO ✓
- Use specific error codes
- Add context to errors
- Include helpful details
- Suggest recovery actions
- Log critical errors
- Handle errors at appropriate level
- Use type-safe error handling

### DON'T ✗
- Swallow errors silently
- Use generic error messages
- Throw strings instead of errors
- Ignore error context
- Over-catch errors
- Create errors without codes
- Use `any` types in TypeScript

## Integration Checklist

### Rust Crate Integration
- [ ] Add `accuscene-errors` to `Cargo.toml`
- [ ] Replace `anyhow::Error` with `AccuSceneError`
- [ ] Use `Result<T>` type alias
- [ ] Add error context to operations
- [ ] Use appropriate error codes
- [ ] Add retry/recovery where needed

### TypeScript Module Integration
- [ ] Import from `@/errors`
- [ ] Wrap root component with `ErrorBoundary`
- [ ] Use `useErrorHandler` in components
- [ ] Replace generic try-catch with typed errors
- [ ] Add error reporting integration
- [ ] Configure error monitoring

## Testing Examples

### Rust
```rust
#[test]
fn test_error_creation() {
    let err = AccuSceneError::validation("Test");
    assert_eq!(err.code(), ErrorCode::Validation);
}

#[tokio::test]
async fn test_retry() {
    let executor = RetryExecutor::new(RetryPolicy::new(3));
    let result = executor.execute(|| async {
        Ok(42)
    }).await;
    assert!(result.is_ok());
}
```

### TypeScript
```typescript
test('error creation', () => {
  const err = AccuSceneError.validation('Test');
  expect(err.code).toBe(ErrorCode.VALIDATION);
});

test('error handler hook', () => {
  const { result } = renderHook(() => useErrorHandler());
  act(() => {
    result.current.handleError(new Error('test'));
  });
  expect(result.current.error).toBeTruthy();
});
```

## Configuration

### ESLint
Use `.eslintrc.enterprise.js` for strict error handling checks:
- No floating promises
- Explicit return types
- No any types
- Strict null checks

### TypeScript
Use `tsconfig.strict.json` for maximum type safety:
- All strict flags enabled
- No implicit any
- No unused variables
- Exact optional properties

---

**Need Help?** See `ERROR_HANDLING_INFRASTRUCTURE.md` for detailed documentation.
