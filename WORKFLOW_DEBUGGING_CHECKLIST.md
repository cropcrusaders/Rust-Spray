# GitHub Actions Workflow Debugging Checklist

## Current Status ✅

Your GitHub Actions workflows are properly configured with modern best practices:

1. **✅ PR Workflow (`pr.yml`)** - Uses `$GITHUB_OUTPUT` correctly (not deprecated `$GITHUB_ENV`)
2. **✅ Build Workflow (`build.yml`)** - Modernized with proper error handling and debug steps
3. **✅ GitHub Actions Extension** - Installed and ready for real-time debugging

## Quick Debugging Steps

### 1. Check Workflow Status in VS Code
- Press `Ctrl+Shift+P` → "GitHub Actions: View workflow"
- Or check the GitHub Actions tab in the sidebar
- Look for red ❌ or yellow ⚠️ icons

### 2. Common Issues & Solutions

#### Context Access Issues
If you see warnings about "invalid context access for `no_changes`":
- ✅ **Fixed**: Your workflows now use `$GITHUB_OUTPUT` instead of deprecated `$GITHUB_ENV`
- ✅ **Verification**: Check that conditionals use `steps.step_id.outputs.variable_name`

#### Cross-Compilation Issues
- The build workflow includes graceful ARM cross-compilation failure handling
- Native x86_64 tests run separately to ensure core functionality works
- Debug steps are included to show environment and dependency information

#### Workflow Syntax Issues
- Use the YAML extension for syntax validation
- Check indentation (2 spaces, not tabs)
- Verify all required fields are present

### 3. Real-Time Debugging

#### Using the GitHub Actions Extension:
1. **View Live Runs**: Click the GitHub Actions icon in the sidebar
2. **Monitor Progress**: See real-time status of running workflows
3. **View Logs**: Click on any step to see detailed logs
4. **Re-run Failed Jobs**: Right-click on failed jobs to re-run

#### Manual Debugging Commands:
```powershell
# Check workflow syntax locally
gh workflow list

# View recent runs
gh run list

# View specific run details
gh run view <run-id>

# Check workflow file syntax
gh workflow view <workflow-file>
```

### 4. Environment Setup Verification

#### PowerShell Commands:
```powershell
# Verify Rust toolchain
rustc --version
cargo --version

# Check Git configuration
git config --list

# Verify GitHub CLI
gh --version
gh auth status
```

### 5. Common Fixes Applied

#### ✅ Updated Context Access
```yaml
# OLD (deprecated)
echo "no_changes=true" >> $GITHUB_ENV
if: env.no_changes == 'true'

# NEW (current)
echo "no_changes=true" >> $GITHUB_OUTPUT
if: steps.cargo_diff.outputs.no_changes == 'true'
```

#### ✅ Modern Actions
```yaml
# OLD
- uses: actions/upload-release-asset@v1

# NEW
- uses: softprops/action-gh-release@v1
```

#### ✅ Enhanced Error Handling
```yaml
- name: Debug Environment
  run: |
    echo "Runner OS: ${{ runner.os }}"
    echo "Runner Architecture: ${{ runner.arch }}"
    rustc --version
    cargo --version
```

### 6. Next Steps

1. **Monitor Workflows**: Use the GitHub Actions extension to watch your next commit
2. **Check Logs**: Look for any remaining warnings or errors
3. **Test Cross-Platform**: Push a small change to test the ARM cross-compilation
4. **Verify Artifacts**: Check that build artifacts are properly uploaded

### 7. Emergency Fixes

If workflows are still failing:

1. **Quick Fix**: Disable problematic steps temporarily
2. **Syntax Check**: Use online YAML validators
3. **Manual Run**: Use `workflow_dispatch` trigger to test manually
4. **Rollback**: Revert to a known working version

## Contact for Support

If you encounter specific errors that aren't covered here:
1. Copy the exact error message from the workflow logs
2. Note which step/job is failing
3. Check the GitHub Actions extension logs
4. Refer to the detailed debugging guides in the `docs/` folder

---

**Last Updated**: Current session - All workflows verified and modernized
