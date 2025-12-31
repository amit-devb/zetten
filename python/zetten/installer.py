import os
import platform
import urllib.request
import shutil
import sys
import hashlib
import importlib.metadata

# This URL always points to the latest released assets
BASE_URL = "https://github.com/amit-devb/zetten/releases/latest/download"

def get_binary_name():
    system = platform.system().lower()
    if system == "linux":
        return "zetten-linux-x86_64"
    if system == "darwin":
        return "zetten-macos-arm64"
    if system == "windows":
        return "zetten-windows-x86_64.exe"
    raise RuntimeError(f"Unsupported platform: {system}")

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()

def download_file(url, dest):
    # GitHub requires a User-Agent or it may return 403/404 for scripts
    req = urllib.request.Request(url, headers={'User-Agent': 'Zetten-Installer'})
    with urllib.request.urlopen(req) as response, open(dest, 'wb') as out_file:
        shutil.copyfileobj(response, out_file)

def install():
    name = get_binary_name()
    binary_url = f"{BASE_URL}/{name}"
    checksum_url = f"{binary_url}.sha256"

    # Define install directory
    if os.name == "nt":
        bin_dir = os.path.join(sys.prefix, "Scripts")
        target = os.path.join(bin_dir, "zetten.exe")
    else:
        bin_dir = os.path.join(sys.prefix, "bin")
        target = os.path.join(bin_dir, "zetten")

    os.makedirs(bin_dir, exist_ok=True)
    tmp_bin = target + ".tmp"
    tmp_sum = tmp_bin + ".sha256"

    try:
        print(f"Downloading Zetten binary...")
        download_file(binary_url, tmp_bin)
        download_file(checksum_url, tmp_sum)

        with open(tmp_sum, "r") as f:
            expected = f.read().split()[0]

        actual = sha256_file(tmp_bin)
        if actual.lower() != expected.lower():
            raise RuntimeError(f"Checksum mismatch! Expected {expected}, got {actual}")

        shutil.move(tmp_bin, target)
        if os.name != "nt":
            os.chmod(target, 0o755)
        
        print(f"✔ Zetten successfully installed to {target}")

    except Exception as e:
        print(f"✘ Installation failed: {e}")
        if "404" in str(e):
            print("Note: The release might still be processing on GitHub. Please wait a minute and try again.")
        if os.path.exists(tmp_bin): os.remove(tmp_bin)
        sys.exit(1)
    finally:
        if os.path.exists(tmp_sum): os.remove(tmp_sum)