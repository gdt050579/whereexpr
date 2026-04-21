# whereexpr

A Rust library for evaluating boolean filter expressions over typed records — like a `WHERE` clause, but for your own types.

---

## What it does

`whereexpr` lets you take a string like:

```
cond_1 && (cond_2 || cond_3)
```

where each named condition is a field-level test like:

```
age > 30
surname is-one-of [Doe, Smith, Williams] {ignore-case}
```

...and evaluate the whole thing against any instance of your struct — at runtime, with no macros.

It separates two concerns cleanly:

- **Conditions** — per-field predicates (`age > 30`, `name starts-with Jo`, `status is active`)
- **Expressions** — boolean combinators over named conditions (`cond_a && !cond_b || (cond_c && cond_d)`)

---

## Quick example

```rust
use whereexpr::*;

struct Person {
    name: String,
    surname: String,
    age: u32,
}

impl Person {
    const NAME: AttributeIndex = AttributeIndex::new(0);
    const SURNAME: AttributeIndex = AttributeIndex::new(1);
    const AGE: AttributeIndex = AttributeIndex::new(2);
}

impl Attributes for Person {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::NAME    => Some(Value::String(&self.name)),
            Self::SURNAME => Some(Value::String(&self.surname)),
            Self::AGE     => Some(Value::U32(self.age)),
            _             => None,
        }
    }

    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::NAME    => Some(ValueKind::String),
            Self::SURNAME => Some(ValueKind::String),
            Self::AGE     => Some(ValueKind::U32),
            _             => None,
        }
    }

    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "name"    => Some(Self::NAME),
            "surname" => Some(Self::SURNAME),
            "age"     => Some(Self::AGE),
            _         => None,
        }
    }
}

fn main() {
    let expr = ExpressionBuilder::<Person>::new()
        .add("is_john",   Condition::from_str("name is John"))
        .add("known_fam", Condition::from_str("surname is-one-of [Doe, Smith, Williams] {ignore-case}"))
        .add("adult_plus",Condition::from_str("age > 30"))
        .build("is_john && known_fam && adult_plus")
        .unwrap();

    let person = Person {
        name: "John".to_string(),
        surname: "doe".to_string(),
        age: 33,
    };

    println!("matches: {}", expr.matches(&person)); // matches: true
}
```

---

## How it works

### 1. Implement `Attributes`

The `Attributes` trait is the bridge between the library and your type. You implement three methods:

| Method | Purpose |
|---|---|
| `get(idx)` | Return the field value for the given index |
| `kind(idx)` | Return the static `ValueKind` for a field index |
| `index(name)` | Map a field name string to its `AttributeIndex` |

### 2. Define conditions

Conditions are named rules, each testing one field of your struct.

**From a string (condition DSL):**
```rust
Condition::from_str("age >= 18")
Condition::from_str("status is-not banned {ignore-case}")
Condition::from_str("score in-range [0, 100]")
```

**Programmatically:**
```rust
let pred = Predicate::with_value(Operation::GreaterThan, 18u32)?;
Condition::new("age", pred)
```

### 3. Build an expression

Register your conditions with `ExpressionBuilder` and pass a boolean expression string:

```rust
let expr = ExpressionBuilder::<MyType>::new()
    .add("rule_a", Condition::from_str("field_x is foo"))
    .add("rule_b", Condition::from_str("field_y > 10"))
    .build("rule_a && !rule_b")
    .unwrap();
```

### 4. Evaluate

```rust
// Panics on type mismatch or missing field
let result: bool = expr.matches(&my_value);

// Returns None on type mismatch or missing field
let result: Option<bool> = expr.try_matches(&my_value);
```

The compiled `Expression` is reusable — build it once and call `matches` on many values.

---

## Virtual attributes

`Attributes::get` does not have to return stored fields — it can return **any computed value**. This lets you filter on derived properties of your type without storing them.

```rust
use whereexpr::*;

struct TextStats {
    text: String,
}

impl TextStats {
    const TEXT:   AttributeIndex = AttributeIndex::new(0);
    const LENGTH: AttributeIndex = AttributeIndex::new(1); // computed, not stored
    const WORDS:  AttributeIndex = AttributeIndex::new(2); // computed, not stored
}

impl Attributes for TextStats {
    fn get(&self, idx: AttributeIndex) -> Option<Value<'_>> {
        match idx {
            Self::TEXT   => Some(Value::String(&self.text)),
            Self::LENGTH => Some(Value::U32(self.text.chars().count() as u32)),
            Self::WORDS  => Some(Value::U32(self.text.split_whitespace().count() as u32)),
            _            => None,
        }
    }
    fn kind(idx: AttributeIndex) -> Option<ValueKind> {
        match idx {
            Self::TEXT   => Some(ValueKind::String),
            Self::LENGTH => Some(ValueKind::U32),
            Self::WORDS  => Some(ValueKind::U32),
            _            => None,
        }
    }
    fn index(name: &str) -> Option<AttributeIndex> {
        match name {
            "text"   => Some(Self::TEXT),
            "length" => Some(Self::LENGTH),
            "words"  => Some(Self::WORDS),
            _        => None,
        }
    }
}

let expr = ExpressionBuilder::<TextStats>::new()
    .add("has_rust",   Condition::from_str("text contains Rust"))
    .add("not_too_long", Condition::from_str("length <= 60"))
    .add("many_words", Condition::from_str("words > 5"))
    .build("has_rust && not_too_long && many_words")
    .unwrap();
```

Computed values are evaluated **lazily** — only when the expression actually reaches the condition that references them.

---

## Condition DSL syntax

A condition string has the form:

```
<attribute> <operation> <value>  [<modifiers>]
```

**Single value:**

```
name is Alice
status is-not active
age > 30
created-at < 1700000000
```

**List value** — one or more comma-separated entries inside `[ ]`:

```
score in-range [1, 100]
role is-one-of [admin, moderator] {ignore-case}
path ends-with-one-of [.log, .tmp]
tag contains-one-of [warn, error, fatal]
```

**Glob with a list** — the glob operation also accepts a list; the attribute matches if it matches **any** of the patterns:

```
full_path glob-match [**/*.rs, **/*.md, **/Cargo.*]
filename  not-glob   [*.tmp, *.bak, *.swp]
```

**DateTime strings** — `DateTime` attributes accept both raw Unix timestamps and
human-readable date strings in `YYYY-MM-DD` format:

```
modified_at > 2024-01-01
created_at  in-range [2020-01-01, 2024-12-31]
expires_at  < 1700000000
```

**Modifiers** appear at the end inside `{...}`:

| Modifier | Effect |
|---|---|
| `{ignore-case}` | Case-insensitive match (strings and paths) |

---

## Operations

| Operation | DSL keywords |
|---|---|
| Equality | `is`, `eq`, `==` |
| Inequality | `is-not`, `neq`, `!=` |
| One of a list | `is-one-of`, `in` |
| Not one of | `is-not-one-of`, `not-in` |
| Starts with | `starts-with` |
| Does not start with | `not-starts-with` |
| Starts with one of | `starts-with-one-of` |
| Does not start with any | `not-starts-with-one-of` |
| Ends with | `ends-with` |
| Does not end with | `not-ends-with` |
| Ends with one of | `ends-with-one-of` |
| Does not end with any | `not-ends-with-one-of` |
| Contains | `contains` |
| Does not contain | `not-contains` |
| Contains one of | `contains-one-of` |
| Contains none of | `not-contains-one-of` |
| Glob pattern match | `glob`, `glob-match` |
| Glob pattern no match | `not-glob`, `not-glob-match` |
| Greater than | `>`, `gt`, `greater-than` |
| Greater than or equal | `>=`, `gte`, `greater-than-or-equal` |
| Less than | `<`, `lt`, `less-than` |
| Less than or equal | `<=`, `lte`, `less-than-or-equal` |
| In range (inclusive) | `in-range` |
| Not in range | `not-in-range` |

---

## Expression syntax

Boolean expressions combine named condition rules:

| Syntax | Meaning |
|---|---|
| `rule_a && rule_b` | AND (`and` also accepted) |
| `rule_a \|\| rule_b` | OR (`or` also accepted) |
| `!rule_a` or `~rule_a` | NOT (`not` also accepted) |
| `(rule_a \|\| rule_b) && rule_c` | Grouping with parentheses |

Conditions:
- conditions names must start with a letter and contain only letters, digits, `_`, and `-`.
- `AND` and `OR` cannot be mixed at the same parenthesis level without explicit grouping.
- Parentheses can be nested up to 8 levels deep.

---

## Supported value types

| `ValueKind` | Rust type | DSL name |
|---|---|---|
| `String` | `&str` | `string` |
| `Path` | `&str` | `path` |
| `Bool` | `bool` | `bool` |
| `U8`–`U64` | `u8`–`u64` | `u8`–`u64` |
| `I8`–`I64` | `i8`–`i64` | `i8`–`i64` |
| `F32`, `F64` | `f32`, `f64` | `f32`, `f64` |
| `Hash128/160/256` | `[u8; N]` | `hash128`, `hash160`, `hash256` |
| `IpAddr` | `std::net::IpAddr` | `ip` |
| `DateTime` | `u64` (Unix timestamp) | `datetime` |

---

## Features

| Feature | Effect |
|---|---|
| `error_description` | Enables `Display` and `description()` on `Error`, `Operation`, and `ValueKind` for human-readable error messages |

Enable in `Cargo.toml`:

```toml
[dependencies]
whereexpr = { version = "0.1", features = ["error_description"] }
```

---

## License

MIT
