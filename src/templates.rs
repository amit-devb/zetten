pub const PYTHON: &str = r#"
[tasks.format]
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
"#;

pub const FASTAPI: &str = r#"
[tasks.format]
cmd = "ruff format app tests"
inputs = ["app/", "tests/"]

[tasks.lint]
cmd = "ruff check app tests"
inputs = ["app/", "tests/"]
depends_on = ["format"]

[tasks.test]
cmd = "pytest"
inputs = ["app/", "tests/"]
depends_on = ["lint"]
allow_exit_codes = [0, 5]

[tasks.run]
cmd = "uvicorn app.main:app --reload"
inputs = ["app/"]
"#;

pub const FLASK: &str = r#"
[tasks.format]
cmd = "ruff format ."
inputs = ["app/", "tests/"]

[tasks.lint]
cmd = "ruff check ."
inputs = ["app/", "tests/"]
depends_on = ["format"]

[tasks.test]
cmd = "pytest"
inputs = ["app/", "tests/"]
depends_on = ["lint"]

[tasks.run]
cmd = "flask run"
inputs = ["app/"]
"#;

pub const DJANGO: &str = r#"
[tasks.format]
cmd = "ruff format ."
inputs = ["."]
    
[tasks.lint]
cmd = "ruff check ."
inputs = ["."]
depends_on = ["format"]

[tasks.test]
cmd = "python manage.py test"
inputs = ["."]
depends_on = ["lint"]

[tasks.migrate]
cmd = "python manage.py migrate"
inputs = ["."]
"#;
