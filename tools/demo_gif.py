#!/usr/bin/env python3
"""
demo_gif.py - dung cac khung man hinh do `ak-hinh` quay thanh GIF dong.

Cach lam giong tools/demo_gif.py cua ak-mcu-base: khung hinh la hinh ma ve cua chinh cac bai
(ak_hinh::ve_*) chay tren may tinh, nen hinh trong GIF la hinh se hien tren OLED cua kit.

  cargo run --release -p ak-hinh --example quay --target x86_64-pc-windows-gnu -- target/quay
  python tools/demo_gif.py target/quay docs/anh

Can Pillow:  pip install pillow
"""

import glob
import os
import sys

from PIL import Image

W, H, PAGES = 128, 64, 8
FRAME_BYTES = W * PAGES
SCALE = 3
BEZEL = 10
FPS = 20                        # example quay.rs ghi 20 khung/giay
BG, OFF, ON = (24, 26, 28), (0, 0, 0), (232, 245, 245)


def frame_image(raw):
    img = Image.new("P", (W, H), 1)
    px = img.load()
    for page in range(PAGES):
        row = raw[page * W:(page + 1) * W]
        for x in range(W):
            col = row[x]
            for bit in range(8):
                if col & (1 << bit):
                    px[x, page * 8 + bit] = 2
    img = img.resize((W * SCALE, H * SCALE), Image.NEAREST)
    out = Image.new("P", (W * SCALE + 2 * BEZEL, H * SCALE + 2 * BEZEL), 0)
    out.putpalette(BG + OFF + ON + (0, 0, 0) * 253)
    out.paste(img, (BEZEL, BEZEL))
    return out


def convert(path, out_dir):
    data = open(path, "rb").read()
    n = len(data) // FRAME_BYTES
    frames = [frame_image(data[i * FRAME_BYTES:(i + 1) * FRAME_BYTES]) for i in range(n)]
    name = os.path.splitext(os.path.basename(path))[0]
    out = os.path.join(out_dir, "%s.gif" % name)
    frames[0].save(out, save_all=True, append_images=frames[1:], duration=1000 // FPS,
                   loop=0, optimize=True, disposal=1)
    print("%-24s %4d khung, %4.1f s -> %s (%d KB)" % (name, n, n / FPS, out, os.path.getsize(out) // 1024))


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    src, out_dir = sys.argv[1], sys.argv[2]
    os.makedirs(out_dir, exist_ok=True)
    files = sorted(glob.glob(os.path.join(src, "*.frames")))
    if not files:
        raise SystemExit("khong co *.frames trong " + src)
    for f in files:
        convert(f, out_dir)


if __name__ == "__main__":
    main()
