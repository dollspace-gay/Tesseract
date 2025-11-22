#!/bin/bash
# Build script for WebAssembly target

set -e

echo "Building Secure Cryptor for WebAssembly..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build with wasm-pack
echo "Building WASM package..."
wasm-pack build --target web --out-dir pkg/web --features console_error_panic_hook

echo "Building WASM package for Node.js..."
wasm-pack build --target nodejs --out-dir pkg/nodejs --features console_error_panic_hook

echo "Building WASM package for bundlers..."
wasm-pack build --target bundler --out-dir pkg/bundler --features console_error_panic_hook

echo ""
echo "✓ WASM build complete!"
echo ""
echo "Output directories:"
echo "  - pkg/web       (for use in browsers via <script>)"
echo "  - pkg/nodejs    (for use in Node.js)"
echo "  - pkg/bundler   (for use with webpack/rollup/etc)"
echo ""
echo "Example usage (web):"
echo "  <script type=\"module\">"
echo "    import init, { encrypt_text } from './pkg/web/secure_cryptor_wasm.js';"
echo "    await init();"
echo "    const encrypted = encrypt_text('password', 'Hello!');"
echo "  </script>"
