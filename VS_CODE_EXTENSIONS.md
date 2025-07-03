# VS Code Extensions for Rust-Spray Development

## Essential Extensions for Rust Development

### Rust-Specific Extensions
```vscode-extensions
rust-lang.rust-analyzer,vadimcn.vscode-lldb,serayuzgur.crates,fill-labs.dependi,swellaby.rust-pack,belfz.search-crates-io
```

### GitHub Actions & CI/CD Debugging
```vscode-extensions
github.vscode-github-actions,redhat.vscode-yaml,github.vscode-pull-request-github,ms-azure-devops.azure-pipelines,gitlab.gitlab-workflow
```

### Additional Useful Extensions
```vscode-extensions
mhutchie.git-graph,eamodio.gitlens,ms-azuretools.vscode-docker,trunk.io,github.codespaces
```

## Extension Descriptions

### Core Rust Development
- **rust-lang.rust-analyzer**: The official Rust language server with IntelliSense, debugging, and refactoring
- **vadimcn.vscode-lldb**: Native debugger based on LLDB for Rust, C, and C++
- **serayuzgur.crates**: Helps manage Rust dependencies in Cargo.toml
- **fill-labs.dependi**: Dependency management for various languages including Rust
- **swellaby.rust-pack**: Extension pack with essential Rust tools
- **belfz.search-crates-io**: Search and insert Rust crates directly from VS Code

### GitHub Actions & CI/CD
- **github.vscode-github-actions**: Official GitHub Actions extension for workflow editing and debugging
- **redhat.vscode-yaml**: YAML language support with GitHub Actions workflow syntax
- **github.vscode-pull-request-github**: GitHub Pull Requests and Issues management
- **ms-azure-devops.azure-pipelines**: Azure Pipelines YAML support
- **gitlab.gitlab-workflow**: GitLab CI/CD workflow support

### Development Tools
- **mhutchie.git-graph**: Visual Git repository graph
- **eamodio.gitlens**: Advanced Git capabilities and history
- **ms-azuretools.vscode-docker**: Docker container management
- **trunk.io**: Universal code quality (linting, formatting, security)
- **github.codespaces**: GitHub Codespaces support

## Installation Notes

1. **Rust Analyzer** is essential for any Rust development
2. **GitHub Actions** extension will help debug your CI/CD workflows
3. **YAML** extension provides syntax highlighting for workflow files
4. **Docker** extension helps with containerized builds
5. **Git Graph** and **GitLens** improve Git workflow visualization

## Usage Tips

- Use **Ctrl+Shift+P** → "GitHub Actions" to access workflow commands
- The **GitHub Actions** extension provides:
  - Syntax highlighting for workflow files
  - Workflow run status in the status bar
  - Direct links to workflow runs
  - Inline error reporting for workflow syntax issues

- **Rust Analyzer** provides:
  - Real-time error checking
  - Code completion
  - Inline documentation
  - Refactoring tools
  - Integrated testing

## CI/CD Debugging

The **GitHub Actions** extension specifically helps with:
- Viewing workflow run history
- Debugging workflow syntax errors
- Monitoring build status
- Quick access to workflow logs
- Inline annotations for workflow issues

This is particularly useful for debugging the cross-compilation issues we've been working on.
