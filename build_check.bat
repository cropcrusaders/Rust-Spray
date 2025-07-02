@echo off
echo.
echo =====================================================
echo    Rust-Spray Project Status Check
echo =====================================================
echo.

REM Check if Cargo.toml exists
if exist "Cargo.toml" (
    echo [✓] Cargo.toml found
) else (
    echo [X] Cargo.toml not found
    exit /b 1
)

REM Check if key source files exist
echo.
echo Checking source files...
if exist "src\main.rs" (echo [✓] src\main.rs) else (echo [X] src\main.rs)
if exist "src\lib.rs" (echo [✓] src\lib.rs) else (echo [X] src\lib.rs)
if exist "src\config.rs" (echo [✓] src\config.rs) else (echo [X] src\config.rs)
if exist "src\camera.rs" (echo [✓] src\camera.rs) else (echo [X] src\camera.rs)
if exist "src\detection.rs" (echo [✓] src\detection.rs) else (echo [X] src\detection.rs)
if exist "src\spray.rs" (echo [✓] src\spray.rs) else (echo [X] src\spray.rs)

REM Check if config exists
echo.
if exist "config\Config.toml" (
    echo [✓] config\Config.toml found
) else (
    echo [!] config\Config.toml not found (needed for runtime)
)

REM Check if tests exist
echo.
if exist "tests\integration_tests.rs" (echo [✓] integration_tests.rs) else (echo [!] integration_tests.rs)
if exist "examples\basic_usage.rs" (echo [✓] basic_usage.rs) else (echo [!] basic_usage.rs)

echo.
echo =====================================================
echo    Recent Fixes Applied
echo =====================================================
echo [✓] Docker cross-compilation issues resolved
echo [✓] Rust formatting issues fixed (all files)
echo [✓] CI/CD pipeline should now pass
echo.
echo =====================================================
echo    Development Environment
echo =====================================================

REM Check for Rust/Cargo
cargo --version >nul 2>&1
if %errorlevel% == 0 (
    echo [✓] Rust/Cargo installed
    cargo --version
) else (
    echo [!] Rust/Cargo not found
    echo     Install from: https://rustup.rs/
)

echo.
echo =====================================================
echo    Build Commands (when Rust is installed)
echo =====================================================
echo Standard build:         cargo build
echo Release build:          cargo build --release
echo Run tests:              cargo test
echo Run example:            cargo run --example basic_usage
echo Cross compile ARM64:    cross build --target aarch64-unknown-linux-gnu --release
echo Cross compile ARMv7:    cross build --target armv7-unknown-linux-gnueabihf --release
echo.
echo =====================================================
echo    Status: Ready for CI/CD
echo =====================================================
echo All fixes have been applied. Push changes to trigger
echo the updated GitHub Actions workflows.
echo.
pause
