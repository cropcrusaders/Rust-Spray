# GitHub Actions Debugging Quick Reference

## Local Testing with act

### Installation (Windows)
```powershell
# Install act via winget
winget install --id nektos.act

# Create alias for convenience
Set-Alias -Name act -Value "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\nektos.act_Microsoft.Winget.Source_8wekyb3d8bbwe\act.exe"
```

### Basic Usage
```powershell
# List all workflows and jobs
act --list

# Run a specific workflow
act workflow_dispatch -W .github/workflows/ci.yml

# Run a specific job
act workflow_dispatch -W .github/workflows/test.yml -j quick-test

# Run with specific event
act push -W .github/workflows/build.yml
```

### Troubleshooting
```powershell
# Check Docker connection
docker --version

# Validate workflow syntax
act --list  # Will show syntax errors

# Use smaller runner images for faster testing
act -P ubuntu-latest=catthehacker/ubuntu:act-latest
```

## VS Code Extensions

### Installed Extensions
- **actionlint** - Real-time workflow linting
- **GitHub Actions** - Syntax highlighting and IntelliSense
- **GitHub Pull Requests** - Integration testing
- **Trunk.io** - Code quality and security scanning
- **YAML** - Enhanced YAML support

### Usage
1. Open workflow files in VS Code
2. Errors will be highlighted in real-time
3. Use Ctrl+Shift+P → "GitHub Actions: Validate Workflow"
4. Check Problems panel for detailed error information

## Common Issues and Solutions

### 1. Duplicate YAML Keys
```yaml
# ❌ Wrong
name: Build
name: Build Again  # Duplicate key error

# ✅ Correct
name: Build
```

### 2. Context Access
```yaml
# ❌ Wrong (deprecated)
- run: echo "value" >> $GITHUB_ENV

# ✅ Correct
- run: echo "value=${{ env.VALUE }}" >> $GITHUB_OUTPUT
```

### 3. Permissions
```yaml
# ❌ Missing permissions
jobs:
  build:
    runs-on: ubuntu-latest

# ✅ With permissions
permissions:
  contents: read
  packages: write
jobs:
  build:
    runs-on: ubuntu-latest
```

### 4. Action Versions
```yaml
# ❌ Outdated
- uses: actions/checkout@v3
- uses: actions/cache@v3

# ✅ Updated
- uses: actions/checkout@v4
- uses: actions/cache@v4
```

## Workflow Testing Strategy

### 1. Syntax Validation
```powershell
# Check YAML syntax
act --list

# Validate in VS Code
# Open workflow file and check Problems panel
```

### 2. Local Testing
```powershell
# Test specific workflow
act workflow_dispatch -W .github/workflows/test.yml

# Test with minimal resources
act -P ubuntu-latest=catthehacker/ubuntu:act-latest
```

### 3. Live Testing
```bash
# Create test branch
git checkout -b test-workflows

# Make small change to trigger workflow
echo "# Test" >> README.md
git add README.md
git commit -m "Test workflow triggers"
git push origin test-workflows

# Monitor workflow runs in GitHub
```

## Performance Optimization

### Cache Keys
```yaml
# ❌ Generic cache key (causes conflicts)
- uses: actions/cache@v4
  with:
    key: rust-cache

# ✅ Unique cache key per workflow
- uses: actions/cache@v4
  with:
    key: rust-cache-${{ github.workflow }}-${{ hashFiles('**/Cargo.lock') }}
```

### Resource Management
```yaml
# Add timeouts to prevent stuck jobs
jobs:
  build:
    runs-on: ubuntu-latest
    timeout-minutes: 30  # Prevent runaway jobs
```

### Error Handling
```yaml
# Add error handling to critical steps
- name: Install dependencies
  run: |
    sudo apt-get update || echo "Update failed, continuing..."
    sudo apt-get install -y build-essential || {
      echo "Failed to install build tools"
      exit 1
    }
```

## Monitoring and Debugging

### GitHub Actions Tab
1. Go to repository → Actions tab
2. Click on workflow runs to see details
3. Check job logs for errors
4. Use re-run options for transient failures

### VS Code Integration
1. Install GitHub Pull Requests extension
2. View workflow status in VS Code
3. Debug workflow issues directly in editor

### Log Analysis
```yaml
# Add debug output to workflows
- name: Debug environment
  run: |
    echo "Runner OS: ${{ runner.os }}"
    echo "GitHub context: ${{ toJson(github) }}"
    echo "Environment variables:"
    env | sort
```

## Best Practices

### 1. Workflow Organization
- Use descriptive names for workflows and jobs
- Group related jobs in the same workflow
- Use consistent naming conventions

### 2. Error Handling
- Always add error handling for external dependencies
- Use fallback strategies for optional features
- Add meaningful error messages

### 3. Security
- Use minimal required permissions
- Pin action versions to specific commits for security
- Avoid exposing secrets in logs

### 4. Performance
- Use caching for dependencies and build artifacts
- Optimize job parallelization
- Set appropriate timeouts

---
*Quick Reference for Rust-Spray GitHub Actions*
*Updated: 2025-07-03*
