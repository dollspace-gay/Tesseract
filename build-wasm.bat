@echo off
REM Build script for WebAssembly target (Windows)

echo Building Secure Cryptor for WebAssembly...

REM Check if wasm-pack is installed
where wasm-pack >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Error: wasm-pack is not installed
    echo Install it with: cargo install wasm-pack
    exit /b 1
)

REM Check if wasm32-unknown-unknown target is installed
rustup target list | findstr /C:"wasm32-unknown-unknown (installed)" >nul
if %ERRORLEVEL% NEQ 0 (
    echo Installing wasm32-unknown-unknown target...
    rustup target add wasm32-unknown-unknown
)

REM Build with wasm-pack
echo Building WASM package for web...
wasm-pack build --target web --out-dir pkg/web --features console_error_panic_hook

echo Building WASM package for Node.js...
wasm-pack build --target nodejs --out-dir pkg/nodejs --features console_error_panic_hook

echo Building WASM package for bundlers...
wasm-pack build --target bundler --out-dir pkg/bundler --features console_error_panic_hook

echo.
echo Build complete!
echo.
echo Output directories:
echo   - pkg\web       (for use in browsers via script)
echo   - pkg\nodejs    (for use in Node.js)
echo   - pkg\bundler   (for use with webpack/rollup/etc)
echo.
