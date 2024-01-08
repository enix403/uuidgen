# `uuidland`

UUID generation and parsing implementation as per [RFC 4122](https://www.ietf.org/rfc/rfc4122.txt).

# Getting Started
<!-- use uuidland::{Uuid, gen, wellknown}; -->

A random V4 UUID can be generated as follows:

```rust
use uuidland::{Uuid, gen};

let uuid: Uuid = gen::v4();
println!("{}", uuid);
```

UUIDs can also pe parsed from strings
```rust
let uuid = Uuid::parse("fe4d0d06-adf3-1fff-bdd3-325096b39f47").unwrap();
```

## Generating UUIDs

This crate supports generating V1, V3, V4 and V5 UUIDs. Use the appropriate functions from `uuidland::gen::*` to generate UUIDs.

- **Time based (V1)**

   ```rust
   use uuidland::gen;
   let uuid_v1 = gen::v1().expect("Failed to generate UUID");
   ```
- **Hash Based (V3 / V5)**

   For versions 3 and 5, a namespace (another UUID) and name are also needed.

   ```rust
   // Some existing UUID
   let namespace = Uuid::parse("3f177ecc-9c78-4e9b-b142-1a8aea0e5624")
   let uuid_v5 = gen::v5(b"foo", Some(namespace));

   // Or use a well-known UUID from uuidland::wellknown module
   use uuidland::wellknown;
   let uuid_v5 = gen::v5(b"bar", wellknown::NS_DNS);

   // Or pass None as namespace. In that case, a random UUID will be
   // used as namespace
   let uuid_v5 = gen::v5(b"bar", None);
   ```

- **Randomly Generated (V4)**

   ```rust
   let uuid_v4 = gen::v4();
   ```