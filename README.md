# uuidgen-rs

A Rust clone of the GNU/util-linux [`uuidgen`](https://man7.org/linux/man-pages/man1/uuidgen.1.html) command-line tool. It supports **all** switches provided by the original.

## Usage

```
Usage:
 uuidgen [options]

Create a new UUID value.

Options:
 -r, --random        generate random-based uuid
 -t, --time          generate time-based uuid
 -n, --namespace ns  generate hash-based uuid in this namespace
 -N, --name name     generate hash-based uuid from this name
 -m, --md5           generate md5 hash
 -s, --sha1          generate sha1 hash
 -x, --hex           interpret name as hex string

 -h, --help       display this help
 -V, --version    display version
```

## Feature parity with util-linux

| Switch | Supported | UUID version |
|--------|-----------|--------------|
| `-r, --random` | ✅ (default) | v4 |
| `-t, --time` | ✅ | v1 |
| `-n, --namespace` | ✅ (`@dns`/`@url`/`@oid`/`@x500` **or** any UUID) | — |
| `-N, --name` | ✅ | — |
| `-m, --md5` | ✅ | v3 |
| `-s, --sha1` | ✅ | v5 |
| `-x, --hex` | ✅ | — |
| `-h, --help` | ✅ | — |
| `-V, --version` | ✅ | — |

## Build & Install

```sh
cargo build --release
# binary is at target/release/uuidgen

cargo install --path .
```

## Examples

```sh
uuidgen                              # random (v4)
uuidgen -r                           # random (v4)
uuidgen -t                           # time-based (v1)
uuidgen -m -n @dns -N www.example.com   # md5 / v3
uuidgen -s -n @dns -N www.example.com   # sha1 / v5
uuidgen -m -n @dns -N 7777772e6578616d706c652e636f6d -x   # name as hex
```

## Test

```sh
cargo test
```

The test suite verifies known v3/v5 values against reference `uuidgen` output.

## License

MIT
