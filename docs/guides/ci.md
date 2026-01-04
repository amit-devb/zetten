# CI Usage

Zetten is designed for the modern CI/CD pipeline. By using Tags and Strict Mode, you can ensure your pipeline is both flexible and safe.


### We just have to add tag to a task
```toml
[tool.zetten.tasks.lint]
cmd = "ruff check src"
inputs = ["src/"]
```

### Run the the task in CI
```bash
zetten run --tag ci
```

### Force a specific version and environment in CI
```bash
zetten run --tag ci -k VERSION=${GITHUB_SHA} -k ENV=prod
```

If a foundational task fails, Zetten halts downstream execution immediately to save CI minutes and prevent cascading failures.


### Practical Example: CI/CD
In a GitHub Action, you might want to pass the commit SHA into your build task:
- Config: `cmd = "echo Building version ${VERSION:-dev}"`
- Local Run: `zetten run build → Output: "Building version dev"`
- CI Run: `zetten run build -k VERSION=${{ github.sha }} → Output: "Building version a1b2c3d...`
