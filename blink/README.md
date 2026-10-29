# Thử firmware Rust trên AK MCU KIT: nháy LED

Chương trình Rust nhỏ nhất chạy được **sau bootloader ak-mcu-base** (không đè bootloader, giữ nguyên OTA cho các
bản C về sau). Mục đích là học và đo: Rust chạy cùng bootloader hiện có được tới đâu, không phải để thay firmware C.

- LED PB8 sáng 0,2 s, tắt 0,8 s; mỗi lần nháy in `nhay lan N` ra console USART1 (PA9/PA10), 115200 8N1.
- Xung nhịp HSI 16 MHz, ghi thẳng thanh ghi theo RM0038, không dùng HAL — để dễ đối chiếu với mã C trong `src/port/stm32l151/` của [ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base).
- Kích thước: 1 604 B flash, 4 B RAM. Ảnh có header: 1 860 B.

> **Trạng thái (29/10/2026):** build được, ảnh qua `mkimage.py info` (status OK). **Chưa chạy trên kit** — lần nạp
> đầu dừng ở bước kết nối vì đổi sang đầu dò ST-Link chưa cập nhật firmware.

## Build và nạp

Cần Rust và target `thumbv7m-none-eabi` (`rustup target add thumbv7m-none-eabi`).

```bash
cd blink
cargo build --release
arm-none-eabi-objcopy -O binary ../target/thumbv7m-none-eabi/release/ak-rust-blink blink.bin
python ../tools/mkimage.py build blink.bin -o blink.img --load-addr 0x08003100 --version 0.1.0 --board ak-l151
python ../tools/mkimage.py info blink.img
```

Nạp vào vùng app (bootloader giữ nguyên):

```bash
STM32_Programmer_CLI -c port=SWD mode=UR -d blink.img 0x08003000 -v -rst
```

Quay về firmware C: nạp lại `app.img` của bản C vào `0x08003000` bằng cùng lệnh.

## Những chỗ phải khớp với bootloader

| | Giá trị | Vì sao |
|---|---|---|
| Bảng vector | `0x08003100` (`memory.x`) | 256 B đầu vùng app là header ảnh; bootloader đặt `VTOR` ở đây |
| Header ảnh | `mkimage.py build`, board `ak-l151` | bootloader kiểm CRC và tên board mỗi lần khởi động, ảnh không header không chạy |
| RAM | bắt đầu `0x20000100` | 52 B đầu RAM là `.noinit` của bootloader (lệnh nhảy, lý do reset); chừa hẳn 256 B |
| Trạng thái chip khi vào app | MSI 2 MHz, watchdog tắt | bootloader nhảy vào app qua một lần reset, nên app tự đặt xung nhịp |

**Bẫy bảng vector.** `cortex-m-rt` mặc định tạo 240 mục ngắt (bảng 1 KB) và đòi bảng căn 1 KB, mà `0x08003100`
chỉ căn 256 B → lỗi link `Interrupt vector table misalignment`. PAC `stm32l1` (feature `stm32l151`) cũng không cứu
được vì khai 57 ngắt của dòng dung lượng lớn. Cách làm ở đây: bật feature `device` của `cortex-m-rt` và tự khai
`__INTERRUPTS` đúng **45 ngắt** của L151CB (Cat.1/2) — 16 + 45 mục = 244 B, vừa căn 256 B. `device.x` để trống vì
chương trình không bật ngắt ngoại vi nào; muốn dùng ngắt thì điền handler vào bảng đó.
