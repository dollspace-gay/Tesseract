# Secure Cryptor - WebAssembly Build

This document describes how to build and use Secure Cryptor as a WebAssembly module for use in web browsers and Node.js.

## Prerequisites

### Required Tools

1. **Rust toolchain** (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **wasm-pack** (for building WASM packages)
   ```bash
   cargo install wasm-pack
   ```

3. **wasm32-unknown-unknown target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Optional Tools

- **wasm-opt** (for optimizing WASM bundles) - included with Binaryen
- **Node.js** (for testing Node.js builds)

## Building

### Quick Start

**Windows:**
```cmd
build-wasm.bat
```

**Linux/macOS:**
```bash
chmod +x build-wasm.sh
./build-wasm.sh
```

### Manual Build

Build for different targets:

```bash
# For web browsers (ES modules)
wasm-pack build --target web --out-dir pkg/web

# For Node.js
wasm-pack build --target nodejs --out-dir pkg/nodejs

# For bundlers (webpack, rollup, etc.)
wasm-pack build --target bundler --out-dir pkg/bundler
```

### Build Options

- `--release` - Build optimized release version (default)
- `--dev` - Build debug version (faster compilation, larger size)
- `--profiling` - Build with profiling enabled
- `-- --features console_error_panic_hook` - Include better panic messages

## Usage

### Web Browser (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Secure Cryptor WASM Example</title>
</head>
<body>
    <script type="module">
        import init, { encrypt_text, decrypt_text, version } from './pkg/web/secure_cryptor.js';

        async function main() {
            // Initialize the WASM module
            await init();

            console.log('Secure Cryptor version:', version());

            // Encrypt some text
            const password = "MySecurePassword123!";
            const plaintext = "Hello, World!";

            const encrypted = encrypt_text(password, plaintext);
            console.log('Encrypted (base64):', encrypted);

            // Decrypt
            const decrypted = decrypt_text(password, encrypted);
            console.log('Decrypted:', decrypted);
        }

        main().catch(console.error);
    </script>
</body>
</html>
```

### Node.js

```javascript
const { encrypt_text, decrypt_text, version } = require('./pkg/nodejs/secure_cryptor.js');

console.log('Secure Cryptor version:', version());

// Encrypt
const encrypted = encrypt_text("password", "Hello, World!");
console.log('Encrypted:', encrypted);

// Decrypt
const decrypted = decrypt_text("password", encrypted);
console.log('Decrypted:', decrypted);
```

### With Module Bundlers (webpack, vite, etc.)

```javascript
import init, {
    encrypt_text,
    decrypt_text,
    encrypt_bytes,
    decrypt_bytes,
    EncryptConfig,
    version
} from 'secure-cryptor-wasm';

async function example() {
    await init();

    // Use custom configuration
    const config = EncryptConfig.secure();

    const encrypted = encrypt_text_with_config(
        "password",
        "Secret message",
        config
    );

    const decrypted = decrypt_text_with_config(
        "password",
        encrypted,
        config
    );

    console.log(decrypted);
}

example();
```

## API Reference

### Text Encryption

#### `encrypt_text(password: string, plaintext: string): string`

Encrypts text with a password using default settings.

- **password**: The encryption password
- **plaintext**: Text to encrypt
- **Returns**: Base64-encoded encrypted data

#### `decrypt_text(password: string, encrypted_base64: string): string`

Decrypts text with a password.

- **password**: The decryption password
- **encrypted_base64**: Base64-encoded encrypted data
- **Returns**: Decrypted plaintext

### Binary Encryption

#### `encrypt_bytes(password: string, data: Uint8Array): Uint8Array`

Encrypts binary data with a password.

- **password**: The encryption password
- **data**: Binary data to encrypt
- **Returns**: Encrypted data (salt || nonce || ciphertext)

#### `decrypt_bytes(password: string, encrypted_data: Uint8Array): Uint8Array`

Decrypts binary data with a password.

- **password**: The decryption password
- **encrypted_data**: Encrypted data
- **Returns**: Decrypted binary data

### Configuration

#### `EncryptConfig`

Configuration object for encryption operations.

**Presets:**
- `EncryptConfig.fast()` - Fast encryption (8MB memory, lower security)
- `EncryptConfig.balanced()` - Balanced (64MB memory, recommended)
- `EncryptConfig.secure()` - Maximum security (128MB memory, slower)

**Custom configuration:**
```javascript
const config = new EncryptConfig();
config.memory_cost = 65536;  // 64MB
config.time_cost = 3;         // 3 iterations
```

### Utility Functions

#### `version(): string`

Returns the library version.

### Security Functions

#### `generate_sri_hash(wasm_bytes: Uint8Array, algorithm: string): string`

Generates Subresource Integrity (SRI) hash for WASM files.

- **wasm_bytes**: The compiled WASM binary
- **algorithm**: Hash algorithm ("sha256", "sha384", or "sha512")
- **Returns**: SRI hash string (e.g., "sha384-...")

**Example**:
```javascript
const response = await fetch('secure_cryptor_bg.wasm');
const wasmBytes = new Uint8Array(await response.arrayBuffer());
const sriHash = generate_sri_hash(wasmBytes, "sha384");
console.log('Use this SRI hash:', sriHash);
```

#### `generate_csp_header(allow_inline_scripts: boolean, allow_eval: boolean, additional_sources?: string[]): string`

Generates Content Security Policy header value for WASM deployment.

- **allow_inline_scripts**: Whether to allow inline scripts
- **allow_eval**: Whether to allow eval() (not recommended)
- **additional_sources**: Additional trusted script sources
- **Returns**: CSP header value string

**Example**:
```javascript
const csp = generate_csp_header(false, false, ["https://cdn.example.com"]);
// Set as Content-Security-Policy header
```

#### `timing_safe_equal(a: Uint8Array, b: Uint8Array): boolean`

Performs constant-time comparison to prevent timing attacks.

- **a**: First byte array
- **b**: Second byte array
- **Returns**: `true` if equal, `false` otherwise

**Example**:
```javascript
const hash1 = new Uint8Array([/* ... */]);
const hash2 = new Uint8Array([/* ... */]);
const equal = timing_safe_equal(hash1, hash2);
```

#### `security_audit_info(): string`

Returns comprehensive security audit information about the WASM module.

**Example**:
```javascript
console.log(security_audit_info());
// Displays security features, mitigations, and recommendations
```

#### `check_security_features(): string`

Checks browser environment for required security features.

- **Returns**: JSON string with feature availability

**Example**:
```javascript
const features = JSON.parse(check_security_features());
console.log('Security features:', features);
```

#### `generate_secure_nonce(length: number): Uint8Array`

Generates cryptographically secure random bytes.

- **length**: Number of bytes (1-1024)
- **Returns**: Secure random bytes

**Example**:
```javascript
const nonce = generate_secure_nonce(12);
// Use for encryption operations
```

## Performance Considerations

### Memory Usage

Argon2 key derivation is memory-intensive:
- **Fast**: ~8MB RAM
- **Balanced**: ~64MB RAM (default)
- **Secure**: ~128MB RAM

Choose based on your target environment:
- Mobile browsers: Use `EncryptConfig.fast()`
- Desktop browsers: Use `EncryptConfig.balanced()` or `EncryptConfig.secure()`

### Bundle Size

Typical WASM bundle sizes:
- **Unoptimized**: ~2-3MB
- **With `wasm-opt -O`**: ~1.5-2MB
- **With `wasm-opt -Oz`**: ~1-1.5MB

To optimize:
```bash
wasm-opt -Oz -o output_optimized.wasm input.wasm
```

### Initialization

Always call `await init()` before using any functions:

```javascript
import init, { encrypt_text } from './pkg/web/secure_cryptor.js';

// ✓ Correct
async function correct() {
    await init();
    return encrypt_text("pass", "data");
}

// ✗ Wrong - will throw error
function wrong() {
    return encrypt_text("pass", "data");  // Error: WASM not initialized
}
```

## Security Considerations

**📚 For comprehensive security documentation, see [WASM_SECURITY.md](WASM_SECURITY.md)**

The WASM_SECURITY.md guide covers:
- Content Security Policy (CSP) configuration
- Subresource Integrity (SRI) implementation
- Side-channel attack mitigations
- Deployment best practices
- Security audit checklist

### Browser Security

- ✅ **WASM memory is isolated** from JavaScript
- ✅ **Sensitive data is zeroized** after use
- ✅ **Side-channel resistant** constant-time operations
- ✅ **Constant-time comparisons** via `timing_safe_equal()`
- ✅ **Subresource Integrity** support via `generate_sri_hash()`
- ⚠️ **Password input should use type="password"** to prevent shoulder-surfing
- ⚠️ **Use HTTPS** to prevent network sniffing
- ⚠️ **Implement CSP** using `generate_csp_header()`

### Web Worker Support

For better security and performance, run crypto operations in a Web Worker:

```javascript
// worker.js
importScripts('./pkg/web/secure_cryptor.js');

self.onmessage = async (e) => {
    const { action, password, data } = e.data;

    if (action === 'encrypt') {
        const result = encrypt_text(password, data);
        self.postMessage({ result });
    }
};

// main.js
const worker = new Worker('worker.js');
worker.postMessage({ action: 'encrypt', password: 'pass', data: 'secret' });
worker.onmessage = (e) => console.log(e.data.result);
```

### Content Security Policy (CSP)

If using CSP, you need to allow WASM:

```html
<meta http-equiv="Content-Security-Policy"
      content="script-src 'self' 'wasm-unsafe-eval';">
```

## Testing

### Unit Tests

Run WASM unit tests:

```bash
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome
```

### Integration Tests

Create an HTML test page:

```html
<!DOCTYPE html>
<html>
<body>
    <script type="module">
        import init, { encrypt_text, decrypt_text } from './pkg/web/secure_cryptor.js';

        async function test() {
            await init();

            const tests = [
                ['password', 'Hello, World!'],
                ['secure123', 'Test data with special chars: €£¥'],
                ['longpass', 'A'.repeat(10000)],
            ];

            for (const [pass, data] of tests) {
                const enc = encrypt_text(pass, data);
                const dec = decrypt_text(pass, enc);

                console.assert(dec === data, `Test failed for: ${data.slice(0, 20)}`);
                console.log(`✓ Test passed: ${data.slice(0, 20)}...`);
            }

            console.log('All tests passed!');
        }

        test().catch(console.error);
    </script>
</body>
</html>
```

## Troubleshooting

### "memory access out of bounds"

- Ensure `await init()` was called
- Check that input data is valid

### "unreachable executed"

- Usually indicates a panic in Rust code
- Enable `console_error_panic_hook` feature for better error messages

### Large bundle size

- Use `wasm-opt -Oz` for size optimization
- Consider code splitting if using bundler
- Use dynamic imports: `const wasm = await import('./pkg/web/secure_cryptor.js');`

### Slow performance

- Use Web Workers for heavy operations
- Consider using `EncryptConfig.fast()` on mobile
- Enable SIMD if browser supports it

## Examples

See `examples/wasm/` directory for complete examples:
- `basic.html` - Simple encryption/decryption
- `file-upload.html` - Encrypt files before upload
- `worker.html` - Using Web Workers
- `node-example.js` - Node.js usage

## License

MIT - See LICENSE file

## Resources

- [wasm-bindgen documentation](https://rustwasm.github.io/wasm-bindgen/)
- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [WebAssembly MDN](https://developer.mozilla.org/en-US/docs/WebAssembly)
