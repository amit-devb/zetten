import os
import platform
import urllib.request
import shutil
import sys

VERSION = "1.0.1"
BASE_URL = "https://github.com/amit-devb/zetten/releases/download"

def get_binary_name():
    system = platform.system().lower()
    arch = platform.machine().lower()

    if system == "linux":
        return "zetten-linux-x86_64"
    if system == "darwin":
        return "zetten-macos-arm64"
    if system == "windows":
        return "zetten-windows-x86_64.exe"

    raise RuntimeError("Unsupported platform")

def install():
    name = get_binary_name()
    url = f"{BASE_URL}/v{VERSION}/{name}"

    target = os.path.join(sys.prefix, "bin", "zetten")
    if os.name == "nt":
        target += ".exe"

    print(f"Downloading {url}")
    with urllib.request.urlopen(url) as r:
        with open(target, "wb") as f:
            shutil.copyfileobj(r, f)

    os.chmod(target, 0o755)
    print("✔ Zetten installed")
