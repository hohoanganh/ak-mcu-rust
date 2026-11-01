#!/usr/bin/env python3
"""Build mot bai, gan header anh cua ak-mcu-base, nap qua ST-Link hoac cong COM.

    python tools/nap.py hello-async              build + nap qua ST-Link
    python tools/nap.py hello-async --com COM3   build + nap qua cong COM (khong can ST-Link)
    python tools/nap.py hello-async --chi-build  chi build, in kich thuoc

ST-Link: anh nap vao 0x08003000 (bootloader giu nguyen), ket noi mode=UR, reset sau khi nap.
Cong COM: `ak_fw.py loader` dua kit vao bootloader (app dang chay - bai Rust hay app C -
phai hieu lenh LOADER), roi `ak_fw.py flash`; anh moi chay thu, tu xac nhan sau 30 s.
Cong cu ARM: bien moi truong ARM_TOOLCHAIN (thu muc bin), roi toolchain cua PlatformIO,
roi PATH. STM32_Programmer_CLI: bien STM32_PROGRAMMER_CLI, roi duong cai mac dinh, roi PATH.
"""

import argparse
import os
import shutil
import subprocess
import sys

GOC = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TARGET = "thumbv7m-none-eabi"
LOAD_ADDR = "0x08003100"
NAP_ADDR = "0x08003000"
BOARD = "ak-l151"


def tim_cong_cu(ten, bien_moi_truong, thu_muc_thu):
    if os.environ.get(bien_moi_truong):
        p = os.environ[bien_moi_truong]
        return p if os.path.isfile(p) else os.path.join(p, ten)
    for d in thu_muc_thu:
        for duoi in ("", ".exe"):
            p = os.path.join(d, ten + duoi)
            if os.path.isfile(p):
                return p
    p = shutil.which(ten)
    if not p:
        sys.exit("khong tim thay %s (dat bien %s)" % (ten, bien_moi_truong))
    return p


def arm(ten):
    return tim_cong_cu("arm-none-eabi-" + ten, "ARM_TOOLCHAIN",
                       [os.path.join(os.path.expanduser("~"), ".platformio", "packages", "toolchain-gccarmnoneeabi", "bin")])


def chay(lenh, **kw):
    print("> " + " ".join(lenh), flush=True)
    r = subprocess.run(lenh, **kw)
    if r.returncode:
        sys.exit(r.returncode)
    return r


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("bai", help="ten thu muc cua bai, vi du hello-async")
    ap.add_argument("--chi-build", action="store_true", help="khong nap")
    ap.add_argument("--com", help="nap qua cong COM nay (vi du COM3, /dev/ttyACM0) thay vi ST-Link")
    a = ap.parse_args()

    chay(["cargo", "build", "--release", "-p", a.bai], cwd=GOC)
    elf = os.path.join(GOC, "target", TARGET, "release", a.bai)
    ra = os.path.join(GOC, "target", "anh")
    os.makedirs(ra, exist_ok=True)
    bin_ = os.path.join(ra, a.bai + ".bin")
    img = os.path.join(ra, a.bai + ".img")

    chay([arm("objcopy"), "-O", "binary", elf, bin_])
    mk = os.path.join(GOC, "tools", "mkimage.py")
    chay([sys.executable, mk, "build", bin_, "-o", img, "--load-addr", LOAD_ADDR,
          "--version", "0.1.0", "--board", BOARD], stdout=subprocess.DEVNULL)
    chay([sys.executable, mk, "info", img])
    chay([arm("size"), elf])
    if a.chi_build:
        print("anh: " + img)
        return

    if a.com:
        fw = os.path.join(GOC, "tools", "ak_fw.py")
        chay([sys.executable, fw, "--port", a.com, "loader"])
        chay([sys.executable, fw, "--port", a.com, "flash", img])
        return

    cli = tim_cong_cu("STM32_Programmer_CLI", "STM32_PROGRAMMER_CLI",
                      [r"C:\Program Files\STMicroelectronics\STM32Cube\STM32CubeProgrammer\bin"])
    r = subprocess.run([cli, "-c", "port=SWD", "mode=UR", "-d", img, NAP_ADDR, "-v", "-rst"],
                       capture_output=True, text=True)
    ok = "Download verified successfully" in r.stdout
    print("nap: " + ("xong, da doc lai khop" if ok else "LOI"))
    if not ok:
        print(r.stdout[-1500:])
        sys.exit(1)


if __name__ == "__main__":
    main()
