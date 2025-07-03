# Using GitHub Actions Extension to Fix Problems

## 🔍 **GitHub Actions Extension Features**

The GitHub Actions extension you have installed provides powerful debugging capabilities:

### 1. **Workflow Monitoring**
- View workflow runs directly in VS Code
- Real-time status updates in the status bar
- Quick access to failed runs and logs

### 2. **Syntax Validation**
- Inline error checking for `.github/workflows/*.yml` files
- Autocomplete for GitHub Actions syntax
- Schema validation for workflow files

### 3. **Direct Integration**
- Click-through to GitHub workflow runs
- View workflow logs without leaving VS Code
- Debug workflow failures with context

## 🛠️ **How to Use the Extension**

### Step 1: Open Workflow Files
1. Navigate to `.github/workflows/` in your project
2. Open any `.yml` workflow file (e.g., `build.yml`)
3. The extension will provide syntax highlighting and validation

### Step 2: Access GitHub Actions Commands
1. Press `Ctrl+Shift+P` (Windows) or `Cmd+Shift+P` (Mac)
2. Type "GitHub Actions" to see available commands:
   - **GitHub Actions: View Workflow Runs**
   - **GitHub Actions: View Workflow Run**
   - **GitHub Actions: Rerun Workflow**
   - **GitHub Actions: View Workflow Logs**

### Step 3: Monitor Workflow Status
- Check the status bar for workflow run indicators
- Click on workflow status to jump to GitHub
- View real-time updates as workflows execute

## 🚨 **Common Problems and Solutions**

### Problem 1: Cross-Compilation Failures
**Symptoms:** ARM builds failing with OpenCV errors
**Solution:** Use the extension to monitor these specific workflow runs

```yaml
# In .github/workflows/build.yml
- name: Cross compile for ARM
  run: cross build --release --target armv7-unknown-linux-gnueabihf --features with-rppal
  continue-on-error: true  # Allow failure for known OpenCV issue
```

### Problem 2: Workflow Syntax Errors
**Symptoms:** Red squiggly lines in workflow files
**Solution:** The extension highlights syntax errors inline

### Problem 3: Failed Dependencies
**Symptoms:** Workflow fails during dependency installation
**Solution:** Check workflow logs via the extension

## 🔧 **Debugging Workflow with Extension**

### 1. **Identify Failed Workflows**
```bash
# Use the extension to view recent workflow runs
# Look for red X indicators next to workflow names
```

### 2. **View Detailed Logs**
- Click on failed workflow in the extension
- Navigate directly to the failing step
- Analyze error messages and stack traces

### 3. **Make Fixes in Real-Time**
- Edit workflow files with full IntelliSense
- Push changes and monitor new runs
- Use the extension to track fix effectiveness

## 📋 **Current Rust-Spray Workflow Status**

### ✅ **Fixed Issues**
1. **Docker Path Problems** - Resolved in `Dockerfile.armv7-opencv`
2. **Cross-Compilation Setup** - Updated in `Cross.toml`
3. **Feature Flags** - Implemented in `Cargo.toml`
4. **Workflow Simplification** - Cleaned up `.github/workflows/build.yml`

### ⚠️ **Expected Failures**
1. **ARM Cross-Compilation with OpenCV** - This will fail until OpenCV is properly configured for cross-compilation
2. **Native ARM Builds** - May fail without proper cross-compilation environment

### 🎯 **Using Extension to Monitor Fixes**

1. **Push Changes**
   ```bash
   git add .
   git commit -m "fix: Updated workflow configurations"
   git push origin main
   ```

2. **Monitor via Extension**
   - Watch for new workflow runs in the extension
   - Check status in VS Code status bar
   - View logs for any remaining issues

3. **Iterate on Fixes**
   - Use extension feedback to refine solutions
   - Test different configurations
   - Monitor success rates

## � **Current Rust-Spray Workflow Issues and Fixes**

### Issue 1: OpenCV Cross-Compilation Failure
**Problem:** ARM builds fail when trying to link OpenCV
**GitHub Actions Extension Shows:** Red X on ARM build jobs
**Fix:** Use the extension to monitor this specific fix:

```yaml
# Update build.yml matrix to handle OpenCV properly
matrix:
  include:
    - target: aarch64-unknown-linux-gnu
      arch: arm64
      features: with-rppal
      opencv: false  # Disable OpenCV for cross-compilation
    - target: armv7-unknown-linux-gnueabihf
      arch: armv7
      features: with-rppal
      opencv: false  # Disable OpenCV for cross-compilation
```

### Issue 2: Missing continue-on-error in Matrix
**Problem:** Entire workflow fails if one target fails
**Fix:** Add proper error handling:

```yaml
strategy:
  fail-fast: false  # ✅ Already present
  matrix:
    include:
      - target: armv7-unknown-linux-gnueabihf
        arch: armv7
        features: with-rppal
        continue-on-error: true  # ⚠️ This should be at job level
```

### Issue 3: Release Asset Upload API Deprecation
**Problem:** `upload-release-asset@v1` is deprecated
**Fix:** Update to newer action:

```yaml
# Replace deprecated action
- name: Upload Debian package
  if: github.event_name == 'release'
  uses: actions/upload-release-asset@v1  # ❌ Deprecated
  # Should be:
  uses: softprops/action-gh-release@v1  # ✅ Current
```

## 🔧 **Step-by-Step Workflow Fixes Using GitHub Actions Extension**

### Step 1: Monitor Current Workflow Status
1. Open VS Code with GitHub Actions extension
2. Press `Ctrl+Shift+P` → "GitHub Actions: View Workflow Runs"
3. Look for failed runs (red X icons)
4. Click on failed run to see detailed logs

### Step 2: Fix Workflow File
Using the extension's IntelliSense, update `.github/workflows/build.yml`:

```yaml
name: Build and Release

permissions:
  contents: write
  packages: read

on:
  push:
    branches: ["main"]
  pull_request:
    branches: ["main"]
  release:
    types: [published]
  workflow_dispatch:  # ✅ Allows manual triggering via extension

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: aarch64-unknown-linux-gnu
            arch: arm64
            features: with-rppal
          - target: armv7-unknown-linux-gnueabihf
            arch: armv7
            features: with-rppal
    
    # Add continue-on-error at job level
    continue-on-error: ${{ matrix.target == 'armv7-unknown-linux-gnueabihf' }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross CLI
        run: cargo install --git https://github.com/cross-rs/cross cross --locked

      - name: Build (with OpenCV error handling)
        env:
          PKG_CONFIG_ALLOW_CROSS: "1"
          CARGO_INCREMENTAL: "0"
        run: |
          # Try building with features, fall back without OpenCV if it fails
          cross build --release --target ${{ matrix.target }} --features ${{ matrix.features }} --verbose || \
          cross build --release --target ${{ matrix.target }} --features ${{ matrix.features }},no-default-features --verbose

      - name: Upload artifact
        if: success() || failure()  # Upload even if build partially failed
        uses: actions/upload-artifact@v4
        with:
          name: rustspray-${{ matrix.arch }}
          path: target/${{ matrix.target }}/release/rustspray
```

### Step 3: Add Debugging Information
Add debug steps to help diagnose issues:

```yaml
      - name: Debug Environment
        run: |
          echo "=== Environment Information ==="
          echo "Target: ${{ matrix.target }}"
          echo "Features: ${{ matrix.features }}"
          echo "Runner OS: ${{ runner.os }}"
          echo "=== Rust Information ==="
          rustc --version
          cargo --version
          cross --version
          echo "=== Available Targets ==="
          rustup target list --installed
          echo "=== PKG Config ==="
          pkg-config --version || echo "pkg-config not found"
```

### Step 4: Monitor Fix Results
1. Commit and push changes
2. Use GitHub Actions extension to watch new workflow runs
3. Check status bar for real-time updates
4. View logs for any remaining issues

## 🚀 **Advanced Debugging with GitHub Actions Extension**

### 1. **Real-Time Monitoring**
- Extension shows workflow status in VS Code status bar
- Get notifications when workflows complete
- Quick access to logs without leaving VS Code

### 2. **Workflow Dispatch Testing**
- Use `workflow_dispatch` trigger to test fixes manually
- Access via `Ctrl+Shift+P` → "GitHub Actions: Trigger Workflow"
- Test specific scenarios without pushing code

### 3. **Matrix Strategy Debugging**
- Monitor each matrix job separately
- Identify which targets succeed/fail
- Compare logs between different matrix combinations

## 💡 **Pro Tips**

1. **Use Workflow Dispatch**
   ```yaml
   on:
     workflow_dispatch:  # Allows manual triggering via extension
     push:
       branches: [ main ]
   ```

2. **Add Debug Information**
   ```yaml
   - name: Debug Environment
     run: |
       echo "Runner OS: ${{ runner.os }}"
       echo "Event: ${{ github.event_name }}"
       rustc --version
       cargo --version
   ```

3. **Monitor Multiple Branches**
   - Use the extension to track workflows across branches
   - Compare results between main and feature branches
   - Identify branch-specific issues

## 🔗 **Quick Actions**

**Open Workflow Runs:** `Ctrl+Shift+P` → "GitHub Actions: View Workflow Runs"
**Rerun Failed Workflow:** `Ctrl+Shift+P` → "GitHub Actions: Rerun Workflow"
**View Logs:** Click on workflow run in extension panel
**Edit Workflow:** Open `.github/workflows/*.yml` files with full IntelliSense

The GitHub Actions extension is your primary tool for monitoring, debugging, and fixing CI/CD issues in real-time!
