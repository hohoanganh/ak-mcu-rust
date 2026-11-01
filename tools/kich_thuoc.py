#!/usr/bin/env python3
"""Do flash/RAM cua moi bai va kiem cac rang buoc khi chay sau bootloader ak-mcu-base.

    cargo build --release
    python tools/kich_thuoc.py            in bang kich thuoc (Markdown)
    python tools/kich_thuoc.py --kiem     in bang, tra ma loi 1 neu co bai vi pham rang buoc

Rang buoc (xem README, muc "Rang buoc chung"):
- bang vector o dung 0x08003100 va <= 256 B (256 B dau vung app la header anh);
- RAM cua app bat dau tu 0x20000100 (dau RAM la .noinit cua bootloader);
- toan bo flash nam trong vung APP (toi 0x08020000).

Doc thang file ELF bang Python, khong can arm-none-eabi-size.
"""

import argparse
import os
import re
import struct
import sys

GOC = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
THU_MUC_BUILD = os.path.join(GOC, "target", "thumbv7m-none-eabi", "release")

VECTOR_ADDR = 0x08003100
VECTOR_TOI_DA = 256
FLASH_DAU, FLASH_CUOI = 0x08000000, 0x08020000
RAM_DAU_APP, RAM_CUOI = 0x20000100, 0x20008000
SHF_ALLOC = 0x2
SHT_NOBITS = 8


def cac_bai():
    """(thu muc, ten file chay) cua moi goi co src/main.rs, theo thu tu trong Cargo.toml goc."""
    goc = open(os.path.join(GOC, "Cargo.toml"), encoding="utf-8").read()
    thanh_vien = re.findall(r'"([^"]+)"', re.search(r"members\s*=\s*\[([^\]]*)\]", goc).group(1))
    for d in thanh_vien:
        if not os.path.isfile(os.path.join(GOC, d, "src", "main.rs")):
            continue
        toml = open(os.path.join(GOC, d, "Cargo.toml"), encoding="utf-8").read()
        yield d, re.search(r'^name\s*=\s*"([^"]+)"', toml, re.M).group(1)


def doc_section(duong_dan):
    """Danh sach (ten, kieu, co, dia_chi, kich_thuoc) cua cac section trong ELF32 little-endian."""
    d = open(duong_dan, "rb").read()
    if d[:4] != b"\x7fELF" or d[4] != 1 or d[5] != 1:
        raise ValueError("khong phai ELF32 little-endian: " + duong_dan)
    shoff, = struct.unpack_from("<I", d, 0x20)
    shentsize, shnum, shstrndx = struct.unpack_from("<HHH", d, 0x2E)
    hang = [struct.unpack_from("<10I", d, shoff + i * shentsize) for i in range(shnum)]
    bang_ten = hang[shstrndx][4]

    def ten(o):
        return d[bang_ten + o:d.index(b"\0", bang_ten + o)].decode()

    return [(ten(h[0]), h[1], h[2], h[3], h[5]) for h in hang]


def do(duong_dan):
    flash = ram = 0
    loi = []
    vector = None
    for ten, kieu, co, dia_chi, kich_thuoc in doc_section(duong_dan):
        if not (co & SHF_ALLOC) or kich_thuoc == 0 and ten != ".vector_table":
            continue
        if ten == ".vector_table":
            vector = (dia_chi, kich_thuoc)
        if FLASH_DAU <= dia_chi < FLASH_CUOI:
            flash += kich_thuoc
            if dia_chi + kich_thuoc > FLASH_CUOI:
                loi.append("%s vuot qua vung APP" % ten)
        elif RAM_DAU_APP - 0x100 <= dia_chi < RAM_CUOI:
            ram += kich_thuoc
            if ten == ".data":
                flash += kich_thuoc  # gia tri dau cua .data nam o flash
            if dia_chi < RAM_DAU_APP:
                loi.append("%s o 0x%08X, de len .noinit cua bootloader" % (ten, dia_chi))
    if vector is None:
        loi.append("khong co .vector_table")
    else:
        if vector[0] != VECTOR_ADDR:
            loi.append("bang vector o 0x%08X, phai o 0x%08X" % (vector[0], VECTOR_ADDR))
        if vector[1] > VECTOR_TOI_DA:
            loi.append("bang vector %d B > %d B" % (vector[1], VECTOR_TOI_DA))
    return flash, ram, (vector[1] if vector else 0), loi


def kb(n):
    return ("%.1f KB" % (n / 1024)).replace(".", ",")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--kiem", action="store_true", help="tra ma loi 1 neu co bai vi pham rang buoc")
    a = ap.parse_args()

    co_loi = False
    print("| Bai | Flash | RAM tinh | Bang vector |")
    print("|---|---:|---:|---:|")
    for thu_muc, ten in cac_bai():
        elf = os.path.join(THU_MUC_BUILD, ten)
        if not os.path.isfile(elf):
            print("| `%s` | chua build | | |" % thu_muc)
            co_loi = True
            continue
        flash, ram, vector, loi = do(elf)
        print("| `%s` | %d B (%s) | %d B (%s) | %d B |" % (thu_muc, flash, kb(flash), ram, kb(ram), vector))
        for l in loi:
            print("  LOI %s: %s" % (thu_muc, l), file=sys.stderr)
            co_loi = True
    if a.kiem and co_loi:
        sys.exit(1)


if __name__ == "__main__":
    main()
