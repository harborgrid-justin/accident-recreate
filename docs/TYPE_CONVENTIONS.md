# AccuScene Enterprise Type Conventions v0.2.5

**Last Updated:** 2025-12-28

## Table of Contents

- [Overview](#overview)
- [TypeScript Type System](#typescript-type-system)
- [Rust Type System](#rust-type-system)
- [Cross-Language Type Mapping](#cross-language-type-mapping)
- [Type Safety Patterns](#type-safety-patterns)
- [Common Pitfalls](#common-pitfalls)
- [Best Practices](#best-practices)

---

## Overview

This document defines type conventions and patterns for AccuScene Enterprise. Proper typing is essential for:

- **Type Safety**: Catching errors at compile time
- **Documentation**: Types serve as inline documentation
- **IDE Support**: Better autocomplete and refactoring
- **Maintainability**: Easier to understand and modify code
- **Performance**: Compiler optimizations based on types

---

## TypeScript Type System

### Primitive Types

```typescript
// Use explicit primitive types
let count: number = 0;
let name: string = "Scene 1";
let isActive: boolean = true;
let timestamp: bigint = 1234567890n;
let id: symbol = Symbol("sceneId");

// Avoid 'any' - use 'unknown' for truly unknown types
function processData(data: unknown): void {
  // Must narrow type before use
  if (typeof data === 'string') {
    console.log(data.toUpperCase());
  }
}

// Use 'never' for impossible cases
function assertNever(x: never): never {
  throw new Error("Unexpected value: " + x);
}

// Use 'void' for functions with no return value
function logMessage(message: string): void {
  console.log(message);
}
```

### Object Types

```typescript
// Prefer interfaces for object shapes
interface Point3D {
  x: number;
  y: number;
  z: number;
}

// Use type aliases for unions, intersections, and utilities
type VehicleType = 'sedan' | 'suv' | 'truck';
type Result<T, E = Error> =
  | { success: true; value: T }
  | { success: false; error: E };

// Extend interfaces
interface ExtendedPoint extends Point3D {
  label?: string;
}

// Intersect types
type TimestampedPoint = Point3D & {
  timestamp: number;
};

// Use readonly for immutable data
interface ImmutableConfig {
  readonly apiUrl: string;
  readonly timeout: number;
}

// Use index signatures for dynamic keys
interface DataMap {
  [key: string]: unknown;
}

// Use mapped types for transformations
type Nullable<T> = {
  [P in keyof T]: T[P] | null;
};

type Optional<T> = {
  [P in keyof T]?: T[P];
};
```

### Array and Tuple Types

```typescript
// Array types - prefer shorthand
const numbers: number[] = [1, 2, 3];
const strings: Array<string> = ['a', 'b', 'c']; // Longer form

// Readonly arrays
const immutableNumbers: readonly number[] = [1, 2, 3];

// Tuples for fixed-length arrays
type Coordinate = [number, number, number];
const position: Coordinate = [1.0, 2.0, 3.0];

// Named tuple elements (TS 4.0+)
type RGB = [red: number, green: number, blue: number];

// Rest elements in tuples
type NumberTuple = [number, ...number[]];

// Optional tuple elements
type OptionalTuple = [string, number?];
```

### Function Types

```typescript
// Function type syntax
type MathOperation = (a: number, b: number) => number;

const add: MathOperation = (a, b) => a + b;

// Function with optional and default parameters
function createScene(
  name: string,
  width: number = 1920,
  height?: number
): Scene {
  // Implementation
}

// Function overloads
function process(value: string): string;
function process(value: number): number;
function process(value: boolean): boolean;
function process(value: string | number | boolean): string | number | boolean {
  return value;
}

// Generic functions
function identity<T>(value: T): T {
  return value;
}

// Constrained generics
function findMax<T extends { value: number }>(items: T[]): T | undefined {
  return items.reduce((max, item) =>
    item.value > (max?.value ?? -Infinity) ? item : max
  , undefined as T | undefined);
}

// Generic with multiple type parameters
function map<T, U>(items: T[], mapper: (item: T) => U): U[] {
  return items.map(mapper);
}
```

### Discriminated Unions

```typescript
// Use discriminated unions for polymorphic types
type Shape =
  | { kind: 'circle'; radius: number }
  | { kind: 'rectangle'; width: number; height: number }
  | { kind: 'triangle'; base: number; height: number };

function area(shape: Shape): number {
  switch (shape.kind) {
    case 'circle':
      return Math.PI * shape.radius ** 2;
    case 'rectangle':
      return shape.width * shape.height;
    case 'triangle':
      return (shape.base * shape.height) / 2;
  }
}

// API responses with discriminated unions
type ApiResponse<T> =
  | { status: 'success'; data: T }
  | { status: 'error'; error: string }
  | { status: 'loading' };

function handleResponse<T>(response: ApiResponse<T>): void {
  switch (response.status) {
    case 'success':
      console.log(response.data);
      break;
    case 'error':
      console.error(response.error);
      break;
    case 'loading':
      console.log('Loading...');
      break;
  }
}
```

### Branded Types

```typescript
// Use branded types for type safety with primitives
type Brand<T, B> = T & { __brand: B };

type SceneId = Brand<string, 'SceneId'>;
type VehicleId = Brand<string, 'VehicleId'>;
type UserId = Brand<string, 'UserId'>;

// Create branded values
function createSceneId(id: string): SceneId {
  return id as SceneId;
}

// Type system prevents mixing
function getScene(id: SceneId): Scene { /* ... */ }
function getVehicle(id: VehicleId): Vehicle { /* ... */ }

const sceneId = createSceneId('scene-123');
const vehicleId = createVehicleId('vehicle-456');

getScene(sceneId); // OK
getScene(vehicleId); // Type error!
```

### Utility Types

```typescript
// Built-in utility types

// Partial - make all properties optional
type PartialScene = Partial<Scene>;

// Required - make all properties required
type RequiredConfig = Required<Config>;

// Readonly - make all properties readonly
type ReadonlyScene = Readonly<Scene>;

// Pick - select specific properties
type SceneBasic = Pick<Scene, 'id' | 'name' | 'description'>;

// Omit - exclude specific properties
type SceneWithoutMetadata = Omit<Scene, 'metadata'>;

// Record - create object type with specific keys and values
type VehicleMap = Record<VehicleId, Vehicle>;

// Extract - extract types from union
type PrimitiveType = Extract<unknown, string | number | boolean>;

// Exclude - exclude types from union
type NonNullable<T> = Exclude<T, null | undefined>;

// ReturnType - extract function return type
type SceneLoaderResult = ReturnType<typeof loadScene>;

// Parameters - extract function parameter types
type LoadSceneParams = Parameters<typeof loadScene>;

// Custom utility types (see global.d.ts)
type DeepPartial<T> = { /* ... */ };
type DeepReadonly<T> = { /* ... */ };
type KeysOfType<T, U> = { /* ... */ };
```

### Template Literal Types

```typescript
// Template literal types (TS 4.1+)
type EventName = `on${Capitalize<string>}`;
type SceneEvent = `scene:${string}`;

// Combining template literals with unions
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';
type Endpoint = `/api/${string}`;
type ApiCall = `${HttpMethod} ${Endpoint}`;

// Intrinsic string manipulation types
type UppercaseEvent = Uppercase<'scene:loaded'>; // "SCENE:LOADED"
type LowercaseEvent = Lowercase<'SCENE:LOADED'>; // "scene:loaded"
type CapitalizedEvent = Capitalize<'scene'>; // "Scene"
type UncapitalizedEvent = Uncapitalize<'Scene'>; // "scene"
```

### Conditional Types

```typescript
// Conditional types
type IsString<T> = T extends string ? true : false;

// Distributed conditional types
type ToArray<T> = T extends unknown ? T[] : never;
type StringOrNumberArray = ToArray<string | number>; // string[] | number[]

// Infer in conditional types
type UnwrapPromise<T> = T extends Promise<infer U> ? U : T;
type Value = UnwrapPromise<Promise<number>>; // number

// Recursive conditional types
type Awaited<T> = T extends Promise<infer U>
  ? Awaited<U>
  : T;

type DeepAwaited = Awaited<Promise<Promise<number>>>; // number
```

---

## Rust Type System

### Primitive Types

```rust
// Integer types
let small: i8 = 127;
let medium: i32 = 2147483647;
let large: i64 = 9223372036854775807;
let unsigned: u32 = 4294967295;
let size: usize = 1024; // Architecture-dependent

// Floating point
let precise: f64 = 3.141592653589793;
let fast: f32 = 3.14159;

// Boolean
let active: bool = true;

// Character (Unicode scalar)
let letter: char = 'A';
let emoji: char = '🚗';

// Unit type
let nothing: () = ();
```

### Compound Types

```rust
// Tuples
let point: (f64, f64, f64) = (1.0, 2.0, 3.0);
let (x, y, z) = point; // Destructuring

// Arrays - fixed size
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
let zeros: [i32; 100] = [0; 100]; // Initialize with same value

// Slices - dynamic view into array
let slice: &[i32] = &numbers[1..3]; // [2, 3]

// Strings
let owned: String = String::from("Hello");
let borrowed: &str = "World";

// Vectors - growable arrays
let mut vehicles: Vec<Vehicle> = Vec::new();
vehicles.push(Vehicle::default());
```

### Structs

```rust
// Named field struct
#[derive(Debug, Clone)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

// Tuple struct
#[derive(Debug, Clone, Copy)]
pub struct SceneId(uuid::Uuid);

// Unit struct
pub struct Marker;

// Generic struct
#[derive(Debug)]
pub struct Container<T> {
    value: T,
}

// Struct with lifetime
pub struct SceneRef<'a> {
    data: &'a SceneData,
    metadata: &'a Metadata,
}

// Struct with constraints
pub struct Processor<T>
where
    T: Serialize + Deserialize,
{
    data: T,
}
```

### Enums

```rust
// Simple enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Sedan,
    Suv,
    Truck,
    Motorcycle,
}

// Enum with data
#[derive(Debug, Clone)]
pub enum SimulationState {
    Idle,
    Running { iteration: usize, time: f64 },
    Paused { at: f64 },
    Completed { result: SimulationResult },
    Error { message: String },
}

// Match on enum
fn handle_state(state: SimulationState) {
    match state {
        SimulationState::Idle => println!("Idle"),
        SimulationState::Running { iteration, time } => {
            println!("Running: iteration {}, time {}", iteration, time);
        }
        SimulationState::Paused { at } => println!("Paused at {}", at),
        SimulationState::Completed { result } => println!("Completed: {:?}", result),
        SimulationState::Error { message } => eprintln!("Error: {}", message),
    }
}

// Generic enum
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

pub enum Option<T> {
    Some(T),
    None,
}
```

### Traits

```rust
// Define a trait
pub trait Simulatable {
    fn update(&mut self, dt: f64);
    fn reset(&mut self);

    // Default implementation
    fn is_active(&self) -> bool {
        true
    }
}

// Implement trait
impl Simulatable for Vehicle {
    fn update(&mut self, dt: f64) {
        self.position += self.velocity * dt;
    }

    fn reset(&mut self) {
        self.position = Vector3::zeros();
        self.velocity = Vector3::zeros();
    }
}

// Generic trait bounds
fn simulate<T: Simulatable>(entity: &mut T, dt: f64) {
    entity.update(dt);
}

// Multiple trait bounds
fn process<T: Serialize + Deserialize + Clone>(data: T) {
    // Implementation
}

// Where clause for complex bounds
fn complex_function<T, U>(t: T, u: U)
where
    T: Serialize + Clone,
    U: Deserialize + Default,
{
    // Implementation
}

// Associated types
pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
}
```

### Type Aliases

```rust
// Simple type alias
pub type Result<T> = std::result::Result<T, Error>;

// Generic type alias
pub type SceneResult<T> = std::result::Result<T, SceneError>;

// Complex type alias
pub type EventCallback = Box<dyn Fn(&Event) -> Result<()> + Send + Sync>;

// Trait object alias
pub type DynSimulatable = Box<dyn Simulatable + Send>;
```

### Newtype Pattern

```rust
// Use newtype for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(uuid::Uuid);

impl SceneId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(uuid::Uuid::parse_str(s)?))
    }

    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

// Implement traits for newtype
impl std::fmt::Display for SceneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<uuid::Uuid> for SceneId {
    fn from(id: uuid::Uuid) -> Self {
        Self(id)
    }
}
```

### Smart Pointers

```rust
// Box - heap allocation
let boxed: Box<Vehicle> = Box::new(Vehicle::default());

// Rc - reference counting (single-threaded)
use std::rc::Rc;
let shared: Rc<Scene> = Rc::new(Scene::default());
let clone = Rc::clone(&shared);

// Arc - atomic reference counting (thread-safe)
use std::sync::Arc;
let shared: Arc<Scene> = Arc::new(Scene::default());

// RefCell - interior mutability (single-threaded)
use std::cell::RefCell;
let mutable: RefCell<i32> = RefCell::new(0);
*mutable.borrow_mut() += 1;

// Mutex - interior mutability (thread-safe)
use std::sync::Mutex;
let shared: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
*shared.lock().unwrap() += 1;

// RwLock - reader-writer lock
use std::sync::RwLock;
let shared: Arc<RwLock<Scene>> = Arc::new(RwLock::new(Scene::default()));
let read = shared.read().unwrap();
```

### Lifetimes

```rust
// Function with lifetime
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}

// Struct with lifetime
pub struct SceneRef<'a> {
    data: &'a SceneData,
}

// Multiple lifetimes
fn compare<'a, 'b>(s1: &'a str, s2: &'b str) -> &'a str
where
    'b: 'a, // 'b outlives 'a
{
    if s1.len() > s2.len() { s1 } else { s1 }
}

// Static lifetime
const VERSION: &'static str = "0.2.5";

// Lifetime elision rules
// These are equivalent:
fn process(s: &str) -> &str { s }
fn process<'a>(s: &'a str) -> &'a str { s }
```

### Phantom Types

```rust
use std::marker::PhantomData;

// Phantom type for compile-time state
pub struct Locked;
pub struct Unlocked;

pub struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    pub fn unlock(self) -> Door<Unlocked> {
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    pub fn lock(self) -> Door<Locked> {
        Door { _state: PhantomData }
    }

    pub fn open(&self) {
        println!("Door opened");
    }
}

// This won't compile:
// let door = Door::<Locked> { _state: PhantomData };
// door.open(); // Error: no method `open` on `Door<Locked>`
```

---

## Cross-Language Type Mapping

### FFI Type Mapping

TypeScript to Rust via N-API:

```typescript
// TypeScript side
interface FFIPoint3D {
  x: number;
  y: number;
  z: number;
}

declare module 'accuscene-core' {
  export function calculateDistance(a: FFIPoint3D, b: FFIPoint3D): number;
}
```

```rust
// Rust side (N-API)
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[napi]
pub fn calculate_distance(a: Point3D, b: Point3D) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}
```

### JSON Serialization

```typescript
// TypeScript
interface Scene {
  id: string;
  name: string;
  vehicles: Vehicle[];
}

const scene: Scene = await fetch('/api/scenes/123').then(r => r.json());
```

```rust
// Rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub vehicles: Vec<Vehicle>,
}

// Serialize
let json = serde_json::to_string(&scene)?;

// Deserialize
let scene: Scene = serde_json::from_str(&json)?;
```

---

## Type Safety Patterns

### Option/Maybe Pattern

```typescript
// TypeScript
type Option<T> = T | null | undefined;

function findById<T extends { id: string }>(
  items: T[],
  id: string
): Option<T> {
  return items.find(item => item.id === id);
}

// Use with optional chaining
const name = findById(scenes, '123')?.name;
```

```rust
// Rust
fn find_by_id<T>(items: &[T], id: &str) -> Option<&T>
where
    T: HasId,
{
    items.iter().find(|item| item.id() == id)
}

// Use with if let or match
if let Some(scene) = find_by_id(&scenes, "123") {
    println!("{}", scene.name());
}
```

### Result/Either Pattern

```typescript
// TypeScript
type Result<T, E = Error> =
  | { success: true; value: T }
  | { success: false; error: E };

function divide(a: number, b: number): Result<number> {
  if (b === 0) {
    return { success: false, error: new Error('Division by zero') };
  }
  return { success: true, value: a / b };
}

// Use with type guards
const result = divide(10, 2);
if (result.success) {
  console.log(result.value);
} else {
  console.error(result.error);
}
```

```rust
// Rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Use with ? operator or match
let result = divide(10.0, 2.0)?;
```

### Builder Pattern

```typescript
// TypeScript
class SceneBuilder {
  private scene: Partial<Scene> = {};

  name(name: string): this {
    this.scene.name = name;
    return this;
  }

  description(description: string): this {
    this.scene.description = description;
    return this;
  }

  build(): Scene {
    if (!this.scene.name) {
      throw new Error('Name is required');
    }
    return this.scene as Scene;
  }
}

const scene = new SceneBuilder()
  .name('Intersection Accident')
  .description('Multi-vehicle collision')
  .build();
```

```rust
// Rust
pub struct SceneBuilder {
    name: Option<String>,
    description: Option<String>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn build(self) -> Result<Scene, BuildError> {
        Ok(Scene {
            name: self.name.ok_or(BuildError::MissingName)?,
            description: self.description,
            ..Default::default()
        })
    }
}

let scene = SceneBuilder::new()
    .name("Intersection Accident")
    .description("Multi-vehicle collision")
    .build()?;
```

---

## Common Pitfalls

### TypeScript

1. **Using `any`**: Defeats type checking
   ```typescript
   // Bad
   function process(data: any) { }

   // Good
   function process(data: unknown) {
     if (typeof data === 'string') {
       // Now TypeScript knows data is string
     }
   }
   ```

2. **Non-null assertion overuse**: Can cause runtime errors
   ```typescript
   // Bad
   const name = scene!.name!.toUpperCase()!;

   // Good
   const name = scene?.name?.toUpperCase() ?? 'Unknown';
   ```

3. **Implicit any in arrays**:
   ```typescript
   // Bad - type is any[]
   const items = [];

   // Good
   const items: Scene[] = [];
   ```

### Rust

1. **Unnecessary cloning**: Impacts performance
   ```rust
   // Bad
   fn calculate(data: Vec<f64>) -> f64 {
       data.clone().iter().sum()
   }

   // Good
   fn calculate(data: &[f64]) -> f64 {
       data.iter().sum()
   }
   ```

2. **Unwrap in library code**: Can panic
   ```rust
   // Bad
   let value = some_option.unwrap();

   // Good
   let value = some_option.ok_or(Error::MissingValue)?;
   ```

3. **String allocations**: Use `&str` when possible
   ```rust
   // Bad
   fn greet(name: String) -> String {
       format!("Hello, {}", name)
   }

   // Good
   fn greet(name: &str) -> String {
       format!("Hello, {}", name)
   }
   ```

---

## Best Practices

### TypeScript

1. **Enable strict mode** in `tsconfig.json`
2. **Use readonly for immutable data**
3. **Prefer unknown over any**
4. **Use branded types for IDs**
5. **Leverage discriminated unions**
6. **Document complex types**

### Rust

1. **Derive common traits** (`Debug`, `Clone`, etc.)
2. **Use newtype pattern for type safety**
3. **Prefer borrowing over ownership**
4. **Use iterators instead of loops**
5. **Implement `From`/`Into` for conversions**
6. **Use `#[must_use]` for important return values**

---

## References

- [TypeScript Deep Dive](https://basarat.gitbook.io/typescript/)
- [Rust Book - Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
- [Rust by Example - Types](https://doc.rust-lang.org/rust-by-example/types.html)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
