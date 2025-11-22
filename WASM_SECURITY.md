# Secure Cryptor - WebAssembly Security Hardening

This document details security hardening measures, best practices, and deployment recommendations for the Secure Cryptor WebAssembly module.

## Table of Contents

1. [Security Architecture](#security-architecture)
2. [Content Security Policy](#content-security-policy)
3. [Subresource Integrity](#subresource-integrity)
4. [Side-Channel Attack Mitigations](#side-channel-attack-mitigations)
5. [Deployment Best Practices](#deployment-best-practices)
6. [Security Audit](#security-audit)

## Security Architecture

### WASM Isolation Benefits

WebAssembly provides strong security guarantees:

- **Memory Isolation**: WASM has its own linear memory space, isolated from JavaScript
- **Sandboxed Execution**: Cannot access system resources without explicit permission
- **Type Safety**: Strongly typed system prevents many common vulnerabilities
- **No Direct DOM Access**: Must go through JavaScript APIs

### Cryptographic Security Features

✓ **AES-256-GCM**: NIST-approved authenticated encryption
✓ **Argon2id**: Memory-hard, GPU-resistant key derivation
✓ **ML-KEM-1024**: Post-quantum key encapsulation
✓ **Constant-Time Operations**: Resistant to timing attacks
✓ **Automatic Zeroization**: Sensitive data cleared from memory

## Content Security Policy

### Why CSP Matters

Content Security Policy prevents:
- Cross-Site Scripting (XSS) attacks
- Unauthorized script execution
- Data injection attacks
- Clickjacking

### Recommended CSP Headers

#### Strict CSP (Recommended for Production)

```http
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'wasm-unsafe-eval';
  object-src 'none';
  base-uri 'self';
  form-action 'self';
  frame-ancestors 'none';
  upgrade-insecure-requests;
```

This policy:
- Allows scripts only from your origin
- Enables WASM execution (`wasm-unsafe-eval`)
- Blocks object embeds (Flash, etc.)
- Prevents clickjacking
- Upgrades all HTTP requests to HTTPS

#### Development CSP

```http
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'unsafe-eval' 'wasm-unsafe-eval';
  object-src 'none';
```

**Note**: `unsafe-eval` is only for development. Remove in production.

### Programmatic CSP Generation

Use the provided helper function:

```javascript
import init, { generate_csp_header } from './pkg/web/secure_cryptor.js';

await init();

// Generate CSP header
const csp = generate_csp_header(
    false,  // allow_inline_scripts
    false,  // allow_eval
    []      // additional_sources
);

console.log('Set this CSP header:', csp);
```

### Implementation Examples

#### Apache (.htaccess)

```apache
Header set Content-Security-Policy "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests;"
```

#### Nginx

```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests;" always;
```

#### Node.js (Express)

```javascript
app.use((req, res, next) => {
  res.setHeader(
    'Content-Security-Policy',
    "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests;"
  );
  next();
});
```

#### HTML Meta Tag (Fallback)

```html
<meta http-equiv="Content-Security-Policy"
      content="default-src 'self'; script-src 'self' 'wasm-unsafe-eval';">
```

**Note**: Meta tags have limitations. Use HTTP headers when possible.

## Subresource Integrity

### What is SRI?

Subresource Integrity ensures that files haven't been tampered with by verifying cryptographic hashes.

### Generating SRI Hashes

#### Using the WASM API

```javascript
import init, { generate_sri_hash } from './pkg/web/secure_cryptor.js';

await init();

// Fetch your WASM file
const response = await fetch('pkg/web/secure_cryptor_bg.wasm');
const wasmBytes = new Uint8Array(await response.arrayBuffer());

// Generate SRI hash (SHA-384 recommended)
const sriHash = generate_sri_hash(wasmBytes, "sha384");
console.log('SRI Hash:', sriHash);
// Output: sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC
```

#### Using Command Line Tools

**OpenSSL**:
```bash
cat pkg/web/secure_cryptor_bg.wasm | openssl dgst -sha384 -binary | openssl base64 -A
```

**Node.js**:
```bash
node -e "const crypto = require('crypto'); const fs = require('fs'); const hash = crypto.createHash('sha384').update(fs.readFileSync('pkg/web/secure_cryptor_bg.wasm')).digest('base64'); console.log('sha384-' + hash);"
```

### Using SRI in HTML

#### For JavaScript Files

```html
<script type="module"
        src="./pkg/web/secure_cryptor.js"
        integrity="sha384-YOUR_HASH_HERE"
        crossorigin="anonymous">
</script>
```

#### For WASM Files (via JavaScript)

```javascript
import init from './pkg/web/secure_cryptor.js';

// Verify WASM integrity before loading
const expectedHash = "sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC";

await init({
  integrity: expectedHash
});
```

### Automated SRI Generation

Create a build script:

```javascript
// generate-sri.js
const fs = require('fs');
const crypto = require('crypto');

function generateSRI(filepath, algorithm = 'sha384') {
  const content = fs.readFileSync(filepath);
  const hash = crypto.createHash(algorithm).update(content).digest('base64');
  return `${algorithm}-${hash}`;
}

const wasmHash = generateSRI('pkg/web/secure_cryptor_bg.wasm');
const jsHash = generateSRI('pkg/web/secure_cryptor.js');

console.log('Update your HTML with these hashes:');
console.log('WASM:', wasmHash);
console.log('JS:', jsHash);

// Optionally write to a manifest file
fs.writeFileSync('sri-manifest.json', JSON.stringify({
  wasm: wasmHash,
  js: jsHash
}, null, 2));
```

Run after building WASM:
```bash
./build-wasm.sh
node generate-sri.js
```

## Side-Channel Attack Mitigations

### Timing Attacks

#### What are Timing Attacks?

Attackers measure execution time to infer secret data. For example, comparing passwords character-by-character leaks information through timing.

#### Our Mitigations

1. **Constant-Time Comparisons**

All sensitive comparisons use constant-time operations:

```javascript
import { timing_safe_equal } from './pkg/web/secure_cryptor.js';

// ✓ SECURE: Constant-time comparison
const passwordHash1 = new Uint8Array([/* hash */]);
const passwordHash2 = new Uint8Array([/* hash */]);
const equal = timing_safe_equal(passwordHash1, passwordHash2);

// ✗ INSECURE: Variable-time comparison (leaks timing info)
// const equal = passwordHash1.toString() === passwordHash2.toString();
```

2. **Memory-Hard KDF**

Argon2id makes timing attacks impractical:
- Forces attackers to use significant memory
- Constant time regardless of password strength
- GPU-resistant

3. **Hardware AES-NI**

Modern CPUs provide constant-time AES instructions, preventing cache-timing attacks.

### Cache-Timing Attacks

#### Mitigations

- Use of AES-NI hardware instructions (constant-time)
- Avoid data-dependent branches in crypto code
- WASM's linear memory reduces cache-line leakage

### Power Analysis Attacks

#### Browser Limitations

Power analysis attacks (measuring power consumption) are not feasible in browser environments:
- JavaScript/WASM cannot access hardware power metrics
- Browser sandboxing prevents such measurements

#### Additional Protections

- Constant-time operations still beneficial
- Prevents attacks if code runs in compromised environments

### Speculative Execution Attacks (Spectre/Meltdown)

#### Browser Mitigations

Modern browsers implement:
- Site Isolation (process-per-origin)
- Reduced timer precision
- SharedArrayBuffer restrictions

#### Our Approach

- Use `SharedArrayBuffer` only when necessary
- Rely on browser's built-in protections
- Constant-time operations reduce exploitability

## Deployment Best Practices

### 1. Always Use HTTPS

**Critical**: Never deploy crypto applications over HTTP.

```nginx
# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}

# HTTPS server
server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    # Modern SSL configuration
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # HSTS
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
}
```

### 2. Implement Security Headers

```nginx
# HSTS
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains; preload" always;

# CSP (see section above)
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'wasm-unsafe-eval';" always;

# X-Frame-Options (prevent clickjacking)
add_header X-Frame-Options "DENY" always;

# X-Content-Type-Options (prevent MIME sniffing)
add_header X-Content-Type-Options "nosniff" always;

# Referrer Policy
add_header Referrer-Policy "strict-origin-when-cross-origin" always;

# Permissions Policy
add_header Permissions-Policy "geolocation=(), microphone=(), camera=()" always;
```

### 3. Use Web Workers

Run crypto operations off the main thread:

**worker.js**:
```javascript
importScripts('./pkg/web/secure_cryptor.js');

const { encrypt_text, decrypt_text } = wasm_bindgen;

async function init() {
  await wasm_bindgen('./pkg/web/secure_cryptor_bg.wasm');
}

self.onmessage = async (e) => {
  await init();

  const { action, password, data } = e.data;

  try {
    if (action === 'encrypt') {
      const result = encrypt_text(password, data);
      self.postMessage({ success: true, result });
    } else if (action === 'decrypt') {
      const result = decrypt_text(password, data);
      self.postMessage({ success: true, result });
    }
  } catch (error) {
    self.postMessage({ success: false, error: error.message });
  }
};
```

**main.js**:
```javascript
const worker = new Worker('worker.js');

worker.postMessage({
  action: 'encrypt',
  password: 'secret',
  data: 'Hello, World!'
});

worker.onmessage = (e) => {
  if (e.data.success) {
    console.log('Encrypted:', e.data.result);
  } else {
    console.error('Error:', e.data.error);
  }
};
```

### 4. Secure Password Input

```html
<!-- Use type="password" to prevent shoulder-surfing -->
<input type="password" id="password" autocomplete="current-password">

<!-- Disable autocomplete for sensitive data -->
<input type="text" id="secretKey" autocomplete="off">
```

### 5. Check Security Features

```javascript
import init, { check_security_features } from './pkg/web/secure_cryptor.js';

await init();

const features = JSON.parse(check_security_features());
console.log('Security features:', features);

// Check for secure context (HTTPS)
if (!window.isSecureContext) {
  alert('Warning: Not running in a secure context (HTTPS required)');
}
```

### 6. Handle Errors Securely

```javascript
try {
  const encrypted = encrypt_text(password, data);
} catch (error) {
  // ✓ GOOD: Generic error message to user
  showError('Encryption failed');

  // Log detailed error for debugging (server-side)
  logError(error);

  // ✗ BAD: Don't show detailed crypto errors to users
  // showError(error.message);  // May leak information
}
```

### 7. Rate Limiting

Implement rate limiting for crypto operations:

```javascript
class RateLimiter {
  constructor(maxAttempts, windowMs) {
    this.maxAttempts = maxAttempts;
    this.windowMs = windowMs;
    this.attempts = [];
  }

  check() {
    const now = Date.now();
    this.attempts = this.attempts.filter(t => now - t < this.windowMs);

    if (this.attempts.length >= this.maxAttempts) {
      return false;  // Rate limit exceeded
    }

    this.attempts.push(now);
    return true;
  }
}

const limiter = new RateLimiter(5, 60000);  // 5 attempts per minute

function decrypt() {
  if (!limiter.check()) {
    alert('Too many attempts. Please wait.');
    return;
  }

  // Proceed with decryption
}
```

## Security Audit

### Running Security Audit

```javascript
import init, { security_audit_info } from './pkg/web/secure_cryptor.js';

await init();

console.log(security_audit_info());
```

Output:
```
Secure Cryptor WASM Security Audit
=====================================

Version: 0.1.0

Security Features:
✓ AES-256-GCM authenticated encryption
✓ Argon2id memory-hard key derivation
✓ Constant-time operations (via subtle crate)
✓ Automatic memory zeroization
✓ Side-channel resistant implementations
✓ Post-quantum cryptography (ML-KEM-1024)

Browser Security:
✓ WASM memory isolation
✓ No eval() usage
✓ Content Security Policy support
✓ Subresource Integrity compatible
✓ Web Worker compatible

Recommendations:
- Always use HTTPS in production
- Implement Content Security Policy
- Use Subresource Integrity for WASM files
- Run crypto operations in Web Workers
- Use type='password' for password inputs
- Enable browser security headers

Side-Channel Mitigations:
- Constant-time comparisons
- Timing-attack resistant KDF
- Cache-timing resistant AES (via hardware AES-NI)
- No data-dependent branching in crypto code
```

### Security Checklist

Before deploying to production:

- [ ] HTTPS enabled with valid certificate
- [ ] Strict Content Security Policy configured
- [ ] Subresource Integrity hashes generated and verified
- [ ] Security headers implemented (HSTS, X-Frame-Options, etc.)
- [ ] Web Workers used for crypto operations
- [ ] Password inputs use `type="password"`
- [ ] Rate limiting implemented
- [ ] Error handling doesn't leak information
- [ ] Browser feature detection implemented
- [ ] Tested in secure context (HTTPS)
- [ ] CSP violations monitored
- [ ] Regular security updates planned

## Additional Resources

### Security Standards

- [NIST Cryptographic Standards](https://csrc.nist.gov/publications/fips)
- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [Web Crypto API Specification](https://www.w3.org/TR/WebCryptoAPI/)

### Browser Security

- [Content Security Policy Level 3](https://www.w3.org/TR/CSP3/)
- [Subresource Integrity](https://www.w3.org/TR/SRI/)
- [Secure Contexts](https://www.w3.org/TR/secure-contexts/)

### Side-Channel Attacks

- [Cache-Timing Attacks on AES](https://cr.yp.to/antiforgery/cachetiming-20050414.pdf)
- [Timing Attacks on Implementations of Diffie-Hellman, RSA, DSS, and Other Systems](https://www.paulkocher.com/doc/TimingAttacks.pdf)

## Reporting Security Issues

If you discover a security vulnerability, please email: security@example.com

**Do not** create public GitHub issues for security vulnerabilities.

## License

MIT - See LICENSE file
