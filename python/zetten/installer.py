from __future__ import annotations

import hashlib
import importlib.metadata
import os
import platform
import shutil
import sys
import tempfile
import urllib.request


GITHUB_REPO = "amit-devb/zetten"


# -------------------------
# Version & URLs
# -------------------------

def get_version() -> str:
    return importlib.metadata.version("zetten")


def base_url() -> str:
    return f"https://github.com/{GITHUB_REPO}/releases/download/v{get_version()}"


# -------------------------
# Platform mapping
# -------------------------

def binary_name() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()

    if system == "linux":
        return "zetten-linux-x86_64"

    if system == "darwin":
        if machine in ("x86_64", "amd64"):
            raise RuntimeError(
                "macOS x86_64 is not supported yet. "
                "Please use Apple Silicon or install via Rosetta."
            )
        return "zetten-macos-arm64"

    if system == "windows":
        return "zetten-windows-x86_64.exe"

    raise RuntimeError(f"Unsupported platform: {system} ({machine})")


# -------------------------
# Paths
# -------------------------

def install_path() -> str:
    if os.name == "nt":
        return os.path.join(sys.prefix, "Scripts", "zetten.exe")
    return os.path.join(sys.prefix, "bin", "zetten")


# -------------------------
# Download helpers
# -------------------------

def download(url: str, dest: str) -> None:
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "zetten-installer"},
    )

    with urllib.request.urlopen(req) as resp:
        ctype = resp.headers.get("Content-Type", "").lower()

        if "text/html" in ctype:
            raise RuntimeError(f"GitHub returned HTML instead of file: {url}")

        with open(dest, "wb") as f:
            shutil.copyfileobj(resp, f)


def sha256(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


# -------------------------
# Install logic
# -------------------------

def install() -> None:
    version = get_version()
    name = binary_name()

    bin_url = f"{base_url()}/{name}"
    sum_url = f"{bin_url}.sha256"

    target = install_path()
    os.makedirs(os.path.dirname(target), exist_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        bin_tmp = os.path.join(tmp, name)
        sum_tmp = bin_tmp + ".sha256"

        print(f"Installing zetten {version}")
        print(f"Downloading {name}...")

        download(bin_url, bin_tmp)
        download(sum_url, sum_tmp)

        with open(sum_tmp) as f:
            expected = f.read().split()[0].strip()

        actual = sha256(bin_tmp)

        if expected.lower() != actual.lower():
            raise RuntimeError(
                "Checksum verification failed\n"
                f"Expected: {expected}\n"
                f"Actual:   {actual}"
            )

        shutil.move(bin_tmp, target)

    if os.name != "nt":
        os.chmod(target, 0o755)

    print(f"✔ zetten installed successfully at {target}")
