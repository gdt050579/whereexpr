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

The same filter can also be written as one rule and compiled with `Expression::from_rule_str`.

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
    const TYPE_ID: u64 = 1;
    const TYPE_NAME: &'static str = "Person";    

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

| Method        | Purpose                                         |
| ------------- | ----------------------------------------------- |
| `get(idx)`    | Return the field value for the given index      |
| `kind(idx)`   | Return the static `ValueKind` for a field index |
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

### 4. Or compile a rule string

`Expression::from_rule_str` takes the conditions and the boolean expression in one string. Each condition value is a condition-DSL string inside single quotes, double quotes, or triple quotes. The `where` block combines those names with `AND`, `OR`, `NOT`, and parentheses.

```rust
let expr = Expression::from_rule_str::<Person>(r#"
rule adults: Person {
    is_john    = "name is John"
    known_fam  = "surname is-one-of [Doe, Smith, Williams] {ignore-case}"
    adult_plus = "age > 30"
} where {
    is_john AND known_fam AND adult_plus
}
"#).unwrap();
```

The type name after `:` is the type the rule applies to. With the `enable_type_check` feature it must equal `T::TYPE_NAME`; otherwise that name is not checked.

`&&`, `||`, and `!` are not accepted in the `where` block. Write `AND`, `OR`, and `NOT`. Whitespace is allowed. A condition name starts with a letter and may then contain letters, digits, `_`, and `-`.

The `from_rule` example filters a small bookshelf with one rule:

```
cd examples
cargo run --example from_rule
```

### 5. Evaluate

```rust
// true only if the expression is definitely true; an unknown result counts as no match
let result: bool = expr.matches(&my_value);

// Some(true) / Some(false), or None if the result is unknown
let result: Option<bool> = expr.evaluate(&my_value);
```

A condition is *unknown* when its attribute is missing (`get` returns `None`). That does not automatically make the whole expression unknown. See [Missing values and three-valued logic](#missing-values-and-three-valued-logic).

Both methods panic if the object is not the type the expression was built for (only checked with the `enable_type_check` feature, see [Features](#features)).

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
    const TYPE_ID: u64 = 2;
    const TYPE_NAME: &'static str = "TextStats";

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

**Regular expressions** — `re-match` tests a `String` or `Path` attribute against a
regex. The pattern **must be single-quoted**: regex syntax uses characters the parser
reserves (`,` separates list entries, a leading `[` opens a list, a trailing `}` opens a
modifier block). Matching is unanchored, so use `^` and `$` for a full match:

```
name     re-match '^report_\d{4}-\d{2}-\d{2}$'
path     re-match '\.log$'
filename not-re-match '^tmp_'
```

Double quotes are rejected — they go through escape processing, which would reject `\d`.
The pattern is compiled when the expression is built, so an invalid regex is reported by
`build()` rather than at match time. `{ignore-case}` is not accepted here; use the inline
`(?i)` flag, which can also be scoped to part of the pattern:

```
message re-match '(?i)\berror\b'
name    re-match '(?i:alice)Smith'
```

**DateTime strings** — `DateTime` attributes accept both raw Unix timestamps and
human-readable date strings in `YYYY-MM-DD` format:

```
modified_at > 2024-01-01
created_at  in-range [2020-01-01, 2024-12-31]
expires_at  < 1700000000
```

**Modifiers** appear at the end inside `{...}`:

| Modifier        | Effect                                     |
| --------------- | ------------------------------------------ |
| `{ignore-case}` | Case-insensitive match (strings and paths) |

`{ignore-case}` applies to the equality, prefix, suffix, substring and membership
operations. It is **rejected** by `re-match` / `not-re-match` — use the inline `(?i)`
flag in the pattern instead — and has no effect on `glob` or on numeric types.

---

## Operations

| Operation               | DSL keywords                         |
| ----------------------- | ------------------------------------ |
| Equality                | `is`, `eq`, `==`                     |
| Inequality              | `is-not`, `neq`, `!=`                |
| One of a list           | `is-one-of`, `in`                    |
| Not one of              | `is-not-one-of`, `not-in`            |
| Starts with             | `starts-with`                        |
| Does not start with     | `not-starts-with`                    |
| Starts with one of      | `starts-with-one-of`                 |
| Does not start with any | `not-starts-with-one-of`             |
| Ends with               | `ends-with`                          |
| Does not end with       | `not-ends-with`                      |
| Ends with one of        | `ends-with-one-of`                   |
| Does not end with any   | `not-ends-with-one-of`               |
| Contains                | `contains`                           |
| Does not contain        | `not-contains`                       |
| Contains one of         | `contains-one-of`                    |
| Contains none of        | `not-contains-one-of`                |
| Has                     | `has`                                |
| Does not have           | `not-has`                            |
| Has one of              | `has-one-of`                         |
| Does not have one of    | `not-has-one-of`                     |
| Glob pattern match      | `glob`, `glob-match`                 |
| Glob pattern no match   | `not-glob`, `not-glob-match`         |
| Regex match             | `re-match`                           |
| Regex no match          | `not-re-match`                       |
| Greater than            | `>`, `gt`, `greater-than`            |
| Greater than or equal   | `>=`, `gte`, `greater-than-or-equal` |
| Less than               | `<`, `lt`, `less-than`               |
| Less than or equal      | `<=`, `lte`, `less-than-or-equal`    |
| In range (inclusive)    | `in-range`                           |
| Not in range            | `not-in-range`                       |

---

## Expression syntax

Boolean expressions combine named condition rules:

| Syntax                           | Meaning                   |
| -------------------------------- | ------------------------- |
| `rule_a && rule_b`               | AND (`and` also accepted) |
| `rule_a \|\| rule_b`             | OR (`or` also accepted)   |
| `!rule_a` or `~rule_a`           | NOT (`not` also accepted) |
| `(rule_a \|\| rule_b) && rule_c` | Grouping with parentheses |

Conditions:
- conditions names must start with a letter and contain only letters, digits, `_`, and `-`.
- `AND` and `OR` cannot be mixed at the same parenthesis level without explicit grouping.
- Parentheses can be nested up to 8 levels deep.

A rule's `where` block, passed to `Expression::from_rule_str`, uses the same names and grouping, but only the words `AND`, `OR`, and `NOT`. `&&`, `||`, and `!` are rejected there.

---

## Missing values and three-valued logic

A condition does not always produce `true` or `false`. If `Attributes::get` returns `None` for the attribute it tests (the field is missing, optional, or cannot be computed for this object), the condition's result is **unknown**.

Expressions combine `true`, `false` and unknown using three-valued (Kleene) logic, the same rules SQL uses for `NULL`:

- **AND**: `false` if any operand is `false`. Otherwise **unknown** if any operand is unknown. Otherwise `true`.
- **OR**: `true` if any operand is `true`. Otherwise **unknown** if any operand is unknown. Otherwise `false`.
- **NOT**: swaps `true` and `false`. Unknown stays unknown.

In other words, an unknown operand only matters when the known operands can't decide the result on their own.

| `a`     | `b`     | `a && b` | `a \|\| b` | `!a`    |
| ------- | ------- | -------- | ---------- | ------- |
| true    | unknown | unknown  | true       | false   |
| false   | unknown | false    | unknown    | true    |
| unknown | unknown | unknown  | unknown    | unknown |

The result does not depend on the order of the operands. `false && unknown` and `unknown && false` are both `false`, and the same holds for any number of operands:

| Expression                                  | Result  |
| ------------------------------------------- | ------- |
| `false && false && unknown && false`        | false   |
| `false && false && unknown && true`         | false   |
| `true && true && unknown && true`           | unknown |
| `false \|\| false \|\| unknown \|\| false` | unknown |
| `false \|\| false \|\| unknown \|\| true`  | true    |

Evaluation still short-circuits: an AND stops at the first `false` and an OR stops at the first `true`. An unknown operand does not stop evaluation, because a later operand may still decide the result.

`matches` and `evaluate` report an unknown result differently:

| Result of the expression | `matches` | `evaluate`    |
| ------------------------ | --------- | ------------- |
| true                     | `true`    | `Some(true)`  |
| false                    | `false`   | `Some(false)` |
| unknown                  | `false`   | `None`        |

`matches` treats unknown as "no match", the same way a SQL `WHERE` clause drops rows whose condition is `NULL`. It never panics because of a missing attribute, so it is safe to use when filtering records with optional fields. Use `evaluate` when you need to tell "does not match" apart from "cannot tell".

```rust
// `age` is an `Option<u32>`, and `get` returns `None` for AGE when it is not set.
let expr = ExpressionBuilder::<Person>::new()
    .add("is_john", Condition::from_str("name is John"))
    .add("over_30", Condition::from_str("age > 30"))
    .build("is_john && over_30")
    .unwrap();

let alice = Person { name: "Alice".into(), age: None };
let john  = Person { name: "John".into(),  age: None };

// The name already makes the AND false, so the missing age does not matter.
expr.matches(&alice);     // false
expr.evaluate(&alice);    // Some(false)

// The name is John, so the missing age decides the result: unknown.
expr.matches(&john);      // false
expr.evaluate(&john);     // None
```

The `three_valued_logic` example checks these rules against a set of expressions and exits with an error if any result is wrong:

```
cd examples
cargo run --example three_valued_logic
```

---

## Supported value types

| `ValueKind`       | Rust type              | DSL name                        |
| ----------------- | ---------------------- | ------------------------------- |
| `String`          | `&str`                 | `string`                        |
| `Path`            | `&str`                 | `path`                          |
| `Bool`            | `bool`                 | `bool`                          |
| `U8`–`U64`        | `u8`–`u64`             | `u8`–`u64`                      |
| `I8`–`I64`        | `i8`–`i64`             | `i8`–`i64`                      |
| `F32`, `F64`      | `f32`, `f64`           | `f32`, `f64`                    |
| `Hash128/160/256` | `[u8; N]`              | `hash128`, `hash160`, `hash256` |
| `IpAddr`          | `std::net::IpAddr`     | `ip`                            |
| `DateTime`        | `u64` (Unix timestamp) | `datetime`                      |

---

## Features

| Feature             | Effect                                                                                                           |
| ------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `error_description` | Enables `Display` and `description()` on `Error`, `Operation`, and `ValueKind` for human-readable error messages |
| `enable_type_check` | `matches` and `evaluate` panic if the object is not the type the expression was built for. `from_rule_str` also returns an error when the rule's type name is not `T::TYPE_NAME` |

Enable in `Cargo.toml`:

```toml
[dependencies]
whereexpr = { version = "0.1", features = ["error_description"] }
```

---

## License

MIT
