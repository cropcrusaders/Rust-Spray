# Enhanced GitHub Actions Debugging Guide

## 🚀 **Upgraded Debugging Stack**

You now have access to a **professional-grade GitHub Actions debugging environment** with the following extensions:

### **Core Debugging Extensions:**

1. **✅ Trunk.io** - Real-time workflow linting with actionlint
2. **✅ actionlint** - Dedicated GitHub Actions workflow validation
3. **✅ GitHub Local Actions** - Run workflows locally for immediate testing
4. **✅ GitHub Pull Requests** - Enhanced GitHub integration
5. **✅ YAML Language Support** - Advanced YAML validation and completion

---

## 🔧 **Enhanced Debugging Workflow**

### **Step 1: Real-Time Workflow Validation**

**actionlint Extension Features:**
- ✅ **Live Syntax Checking**: Errors appear as you type
- ✅ **Security Validation**: Catches unsafe patterns
- ✅ **Best Practices**: Enforces GitHub Actions best practices
- ✅ **Auto-Fix Suggestions**: Provides quick fixes for common issues

**How to Use:**
1. Open any `.github/workflows/*.yml` file
2. actionlint will automatically validate your workflow
3. Red squiggles indicate errors, yellow for warnings
4. Hover over errors for detailed explanations

### **Step 2: Local Workflow Testing** ⭐ **Game Changer!**

**GitHub Local Actions Features:**
- 🏠 **Run Locally**: Test workflows without pushing to GitHub
- 🚀 **Instant Feedback**: See results immediately
- 💰 **Save CI Minutes**: No need to waste GitHub Actions minutes on debugging
- 🔍 **Step-by-Step Debugging**: Debug individual workflow steps

**How to Use:**
1. Press `Ctrl+Shift+P` → "GitHub Local Actions: Run Workflow"
2. Select your workflow file
3. Choose which jobs/steps to run
4. Watch real-time execution in the terminal

### **Step 3: Advanced Trunk.io Integration**

**Trunk.io Features for GitHub Actions:**
- 🔍 **Multi-Tool Validation**: Runs actionlint + additional security checks
- 📊 **Quality Metrics**: Tracks workflow quality over time
- 🔐 **Security Scanning**: Detects credential leaks and security issues
- 📝 **Formatting**: Auto-formats YAML files

**How to Use:**
1. Trunk.io automatically runs on workflow files
2. View results in the Problems panel (`Ctrl+Shift+M`)
3. Use `Ctrl+Shift+P` → "Trunk: Check All" for comprehensive analysis

### **Step 4: Enhanced GitHub Integration**

**GitHub Pull Requests Extension:**
- 📊 **Workflow Status**: See workflow results directly in PR view
- 🔄 **Re-run Workflows**: Restart failed workflows from VS Code
- 📝 **PR Comments**: View workflow-related PR comments
- 🔍 **Detailed Logs**: Access workflow logs without leaving VS Code

---

## 🛠️ **Debugging Commands**

### **Local Testing Commands:**

```powershell
# Install act (GitHub Actions local runner)
# Run this in PowerShell as Administrator:
winget install nektos.act

# Or using Chocolatey:
choco install act-cli

# Test a specific workflow locally:
act -W .github/workflows/build.yml

# Test with specific events:
act push
act pull_request
act workflow_dispatch
```

### **actionlint Commands:**

```powershell
# Install actionlint globally (optional - extension includes it)
go install github.com/rhymond/actionlint/cmd/actionlint@latest

# Run actionlint manually:
actionlint .github/workflows/build.yml

# Check all workflows:
actionlint .github/workflows/*.yml
```

### **VS Code Commands:**

```
Ctrl+Shift+P → "GitHub Local Actions: Run Workflow"
Ctrl+Shift+P → "GitHub Pull Requests: Refresh"
Ctrl+Shift+P → "Trunk: Check All"
Ctrl+Shift+P → "YAML: Validate Document"
```

---

## 🎯 **Advanced Debugging Scenarios**

### **Scenario 1: Workflow Syntax Issues**

**Before (Basic Extension):**
- Limited error detection
- No real-time validation
- Had to push to GitHub to see errors

**After (Enhanced Stack):**
- ✅ **Real-time validation** as you type
- ✅ **Detailed error messages** with solutions
- ✅ **Auto-completion** for GitHub Actions syntax
- ✅ **Security issue detection**

### **Scenario 2: Cross-Platform Testing**

**Enhanced Local Testing:**
```powershell
# Test on different platforms locally
act -P ubuntu-latest=catthehacker/ubuntu:act-latest
act -P windows-latest=catthehacker/windows:act-latest
act -P macos-latest=catthehacker/macos:act-latest
```

### **Scenario 3: Secret and Environment Variable Testing**

**Local Testing with Secrets:**
```powershell
# Create .secrets file for local testing
echo "GITHUB_TOKEN=your_token_here" > .secrets

# Run with secrets
act -s GITHUB_TOKEN
```

### **Scenario 4: Debugging Specific Jobs**

**Target Specific Jobs:**
```powershell
# Run only the test job
act -j test-native

# Run only ARM cross-compilation
act -j build-arm
```

---

## 📊 **Debugging Dashboard**

### **VS Code Panels to Monitor:**

1. **Problems Panel** (`Ctrl+Shift+M`)
   - actionlint errors and warnings
   - Trunk.io quality issues
   - YAML syntax problems

2. **Terminal Panel** (`Ctrl+``)
   - Local workflow execution
   - actionlint output
   - GitHub Actions logs

3. **GitHub Actions Panel** (Sidebar)
   - Workflow run status
   - Job results
   - Artifact downloads

4. **Source Control Panel** (`Ctrl+Shift+G`)
   - Workflow changes
   - PR status
   - Commit history

---

## 🔍 **Troubleshooting Guide**

### **Common Issues and Solutions:**

#### **Issue: actionlint not working**
**Solution:**
```powershell
# Check actionlint installation
actionlint --version

# Reinstall if needed
go install github.com/rhymond/actionlint/cmd/actionlint@latest
```

#### **Issue: Local Actions failing**
**Solution:**
```powershell
# Check act installation
act --version

# Update act
winget upgrade nektos.act

# Use verbose mode for debugging
act -v
```

#### **Issue: Workflow validation errors**
**Solution:**
1. Check the Problems panel for specific errors
2. Use `Ctrl+Shift+P` → "YAML: Validate Document"
3. Verify GitHub Actions syntax with actionlint
4. Test locally with GitHub Local Actions

---

## 🚀 **Best Practices with Enhanced Stack**

### **Development Workflow:**

1. **Write Workflow** → Real-time validation with actionlint
2. **Test Locally** → Use GitHub Local Actions for immediate feedback
3. **Quality Check** → Run Trunk.io for comprehensive analysis
4. **Commit** → Push with confidence
5. **Monitor** → Use enhanced GitHub integration for real-time status

### **Debugging Workflow:**

1. **Identify Issue** → Check Problems panel for specific errors
2. **Test Locally** → Use GitHub Local Actions to reproduce
3. **Fix Iteratively** → Make changes and test locally
4. **Validate** → Use actionlint to ensure correctness
5. **Deploy** → Push to GitHub with confidence

---

## 📋 **Quick Reference**

### **Essential Keyboard Shortcuts:**
- `Ctrl+Shift+P` → Command palette
- `Ctrl+Shift+M` → Problems panel
- `Ctrl+`` → Terminal panel
- `Ctrl+Shift+G` → Source control panel

### **Key Extensions Status:**
- ✅ **actionlint** - Real-time workflow validation
- ✅ **GitHub Local Actions** - Local workflow testing
- ✅ **GitHub Pull Requests** - Enhanced GitHub integration
- ✅ **Trunk.io** - Comprehensive code quality
- ✅ **YAML Language Support** - Advanced YAML features

---

## 🎉 **You Now Have Professional-Grade GitHub Actions Debugging!**

This enhanced stack provides:
- **Real-time validation** as you type
- **Local testing** without GitHub Actions minutes
- **Security scanning** and best practices enforcement
- **Professional workflow management** tools
- **Comprehensive error detection** and solutions

**Next Steps:**
1. Open a workflow file to see actionlint in action
2. Try running a workflow locally with GitHub Local Actions
3. Use the enhanced GitHub integration for PR management
4. Monitor the Problems panel for quality insights

You're now equipped with the same debugging tools used by professional DevOps engineers!
