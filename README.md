# Satch

High-performance glob pattern matching for Rust and CLI.

## Install

### CLI

```bash
cargo install satch
```

### Library

```toml
[dependencies]
satch = "0.1.0"
```

## CLI Usage

### Pattern Matching

```bash
# Basic matching
echo "src/main.rs" | satch "*.rs"              # NO MATCH
echo "src/main.rs" | satch --basename "*.rs"   # MATCH

# Multiple paths
satch "*.rs" src/main.rs lib.rs test.js        # Test multiple files
```

### File Listing

```bash
# Current directory
satch --list "*.rs"                             # List matching files

# Recursive search
satch --list --recursive --basename "*.js"     # Find all .js files
```

### Advanced Patterns

```bash
# Globstar patterns
satch --list --recursive "**/test/**/*.js"     # Find test files

# Character classes
satch --basename "[a-z]*.txt" file1.txt File2.txt  # Lowercase names only
```

## Library Usage

### Basic Matching

```rust
use satch::is_match;

// Simple patterns
is_match("file.js", "*.js");          // true
is_match("src/main.rs", "*.rs");      // false

// With basename matching needed
is_match("main.rs", "*.rs");          // true
```

### Advanced Patterns

```rust
// Globstars
is_match("src/lib/utils.rs", "**/*.rs");       // true
is_match("deep/nested/file.js", "**/file.js"); // true

// Character classes
is_match("test1.js", "test[0-9].js");          // true
is_match("file.txt", "[a-z]*.txt");            // true
is_match("File.txt", "[a-z]*.txt");            // false (case sensitive)
```

## Patterns

| Pattern       | Example Matches                   |
| ------------- | --------------------------------- |
| `*.js`        | `main.js`, `test.js`              |
| `**/*.js`     | `src/main.js`, `lib/test.js`      |
| `src/**/*.rs` | `src/lib/main.rs`, `src/utils.rs` |
| `[a-z]*.txt`  | `file.txt`, `readme.txt`          |

## Credits

Rust port of [micromatch](https://github.com/micromatch/micromatch) and [picomatch](https://github.com/micromatch/picomatch).

## License

MIT
