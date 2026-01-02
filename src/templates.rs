pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

pub const TEMPLATES: &[Template] = &[
    Template {
        name: "python",
        description: "Standard Python (Ruff, Pytest, Release automation)",
        content: PYTHON,
    },
    Template {
        name: "fastapi",
        description: "FastAPI web project (Uvicorn, Ruff, Pytest)",
        content: FASTAPI,
    },
    Template {
        name: "flask",
        description: "Flask web project (Dev server, Ruff, Pytest)",
        content: FLASK,
    },
    Template {
        name: "django",
        description: "Django project (Manage.py, Migrations, Ruff)",
        content: DJANGO,
    },
];

pub const PYTHON: &str = r#"[tasks.format]
cmd = "ruff format ."
inputs = ["src/", "tests/"]

[tasks.lint]
cmd = "ruff check ."
inputs = ["src/", "tests/"]
depends_on = ["format"]

[tasks.test]
cmd = "pytest"
inputs = ["src/", "tests/"]
depends_on = ["lint"]
allow_exit_codes = [0, 5]

[tasks.release]
description = "Bump version and tag (usage: zetten run release -- v1.0.0)"
cmd = """
python3 -c "import re; p=re.sub(r'version = \".*\"', 'version = \"$1\"', open('pyproject.toml').read()); open('pyproject.toml', 'w').write(p)" && \
git add pyproject.toml && \
git commit -m "chore: bump version to $1" && \
git tag $1 && \
git push origin main --tags
"""
inputs = ["pyproject.toml"]
"#;

pub const FASTAPI: &str = r#"[tasks.run]
cmd = "uvicorn app.main:app --reload"
inputs = ["app/"]

[tasks.test]
cmd = "pytest"
inputs = ["app/", "tests/"]
"#;

pub const FLASK: &str = r#"[tasks.run]
cmd = "flask run"
inputs = ["app/"]
"#;

pub const DJANGO: &str = r#"[tasks.migrate]
cmd = "python manage.py migrate"
inputs = ["."]

[tasks.test]
cmd = "python manage.py test"
inputs = ["."]
"#;

pub const DEFAULT: &str = r#"[tasks.hello]
cmd = "echo 'Hello from Zetten!'"
"#;

pub fn format_for_pyproject(template: &str) -> String {
    let mut output = String::from("\n[tool.zetten.tasks]\n");
    for line in template.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[tasks.") {
            let new_line = line.replace("[tasks.", "[tool.zetten.tasks.");
            output.push_str(&new_line);
        } else if !trimmed.is_empty() {
            output.push_str(line);
        }
        output.push('\n');
    }
    output
}