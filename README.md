The `regex_with` library provides Rust procedural macros to enable regex-based parsing capabilities for custom types. It simplifies the integration of regex patterns into the parsing process of data structures.

## Example

```rust
use regex_with::{Capturable, de::FromStr};
#[derive(serde::Deserialize, Capturable, FromStr)]
#[regex_with(re = "^(?P<id>\\d+)$")]
struct Record {
    id: u32,
}

let record: Record = "123".parse().unwrap();
assert_eq!(record.id, 123);
```
