# Tracey Integration Status

## Current State (December 31, 2024)

We have successfully integrated [tracey](https://github.com/bearcove/tracey) for specification coverage tracking. Due to a critical bug in the published v1.0.0 release on crates.io, we build tracey from source.

## ⚠️ Important: Build from Source Required

The tracey v1.0.0 release on crates.io is **broken** - it cannot scan files for markers. Always build from source until this is fixed upstream.

### Building Tracey from Source

```bash
# Clone the repository
git clone https://github.com/bearcove/tracey /tmp/tracey --depth 1
cd /tmp/tracey

# Build the release binary
cargo build --release

# Install to your cargo bin directory
cp target/release/tracey ~/.cargo/bin/

# Verify installation
tracey --help
```

## Marker Syntax

Tracey requires markers to include the spec name prefix. In our project, all markers must use the `web-app` prefix (matching the spec name in config.kdl):

### Implementation Markers (Rust/TypeScript)
```rust
// Correct format - includes spec name prefix
/// web-app[impl auth.session.creation]
/// web-app[impl auth.session.cookie.name]
pub struct CurrentSession { ... }
```

### Verification Markers (E2E Tests)
```typescript
/**
 * web-app[verify auth.oauth.success.redirect]
 * web-app[verify auth.protected.extraction]
 */
test('successful OAuth login', async () => { ... });
```

### Incorrect Formats (Won't Be Detected)
```rust
// Missing spec name prefix - WON'T WORK
/// [impl auth.session.creation]      ❌
/// [verify auth.protected.extraction] ❌
```

## Coverage Statistics

As of December 31, 2024:
- **Total Rules**: 194 requirements in `specs/web_app/*.md`
- **Coverage**: 29.9% (58 rules implemented)
- **Implementation Markers**: 143+ `web-app[impl]` markers
- **Verification Markers**: Multiple `web-app[verify]` markers in E2E tests

### Coverage Breakdown
- **Auth**: 35 rules with implementations in `server/src/routes/auth.rs`
- **Battlesnakes**: 56 rules with CRUD operation implementations
- **Games**: 76 rules with game management implementations
- **Profiles**: 27 rules with profile route implementations

## Running Tracey Locally

After building from source:

```bash
# Extract rules from specifications
tracey --config .config/tracey/config.kdl rules specs/web_app/*.md

# Generate coverage matrix (HTML format)
tracey matrix --config .config/tracey/config.kdl -f html > tracey-report.html

# Generate coverage matrix (JSON format)
tracey matrix --config .config/tracey/config.kdl -f json > tracey-report.json

# Generate text summary
tracey matrix --config .config/tracey/config.kdl -f text

# Check what rules a specific line implements
tracey at server/src/routes/auth.rs:27

# Find all implementations of a specific rule
tracey impact auth.session.creation
```

## CI Integration

The GitHub Actions workflow (`.github/workflows/tracey.yml`) automatically:
1. Builds tracey from source on every run
2. Generates coverage reports in HTML and JSON formats
3. Uploads reports as artifacts
4. Displays coverage summary in CI logs

**Note**: The CI workflow builds tracey from source because the crates.io release is broken.

## Configuration

Location: `.config/tracey/config.kdl`
```kdl
spec {
  name "web-app"  # This is the prefix required in all markers
  rules_glob "specs/web_app/**/*.md"
  include "server/src/**/*.rs"
  include "e2e/tests/**/*.ts"
  exclude "target/**"
  exclude "node_modules/**"
  exclude "e2e/node_modules/**"
}
```

## Adding New Markers

When implementing a requirement:

1. Find the rule ID in the spec files (e.g., `r[auth.session.creation]`)
2. Add the implementation marker with the `web-app` prefix:
   ```rust
   /// web-app[impl auth.session.creation]
   pub fn create_session() { ... }
   ```

When writing tests that verify requirements:

1. Add verification markers with the `web-app` prefix:
   ```typescript
   /**
    * web-app[verify auth.session.creation]
    */
   test('creates new session', async () => { ... });
   ```

## Known Issues

1. **crates.io Release Broken**: The v1.0.0 release on crates.io doesn't scan files. Must build from source.
2. **Prefix Requirement**: All markers must include the spec name prefix (`web-app`). Markers without the prefix are silently ignored.
3. **Case Sensitivity**: Rule IDs are case-sensitive. Ensure exact matches between specs and markers.

## Future Improvements

1. File bug report upstream about broken crates.io release
2. Continue adding [impl] markers to improve coverage percentage
3. Add more [verify] markers to E2E tests for requirement verification
4. Consider automating marker addition with code generation