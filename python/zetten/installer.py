import os
import platform
import urllib.request
import shutil
import sys
import hashlib
import importlib.metadata

VERSION = importlib.metadata.version("zetten")
BASE_URL = "https://github.com/amit-devb/zetten/releases/download"

def get_binary_name():
    system = platform.system().lower()

    if system == "linux":
        return "zetten-linux-x86_64"
    if system == "darwin":
        return "zetten-macos-arm64"
    if system == "windows":
        return "zetten-windows-x86_64.exe"

    raise RuntimeError("Unsupported platform")

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()

def install():
    name = get_binary_name()
    binary_url = f"{BASE_URL}/v{VERSION}/{name}"
    checksum_url = f"{binary_url}.sha256"

    bin_dir = os.path.join(sys.prefix, "bin")
    os.makedirs(bin_dir, exist_ok=True)

    target = os.path.join(bin_dir, "zetten")
    if os.name == "nt":
        target += ".exe"

    tmp_bin = target + ".tmp"
    tmp_sum = tmp_bin + ".sha256"

    urllib.request.urlretrieve(binary_url, tmp_bin)
    urllib.request.urlretrieve(checksum_url, tmp_sum)

    with open(tmp_sum) as f:
        expected = f.read().split()[0]

    actual = sha256_file(tmp_bin)
    if actual != expected:
        raise RuntimeError("Checksum mismatch")

    shutil.move(tmp_bin, target)
    os.chmod(target, 0o755)
    os.remove(tmp_sum)

    print("✔ Zetten installed and verified")
