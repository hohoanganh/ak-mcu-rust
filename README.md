# ak-mcu-rust — thử firmware Rust trên AK MCU KIT

Các thử nghiệm viết firmware bằng Rust cho AK MCU KIT (STM32L151CB: 128 KB flash, 16 KB RAM, 32 MHz), chạy **sau
bootloader của [ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base)**: không đè bootloader, nên kit vẫn quay về
firmware C bất cứ lúc nào bằng cách nạp lại `app.img` của bản C.

Đây là chỗ học và đo xem Rust làm được tới đâu trên chip nhỏ, không phải bản thay thế cho ak-mcu-base.

| Thư mục | Nội dung | Trạng thái |
|---|---|---|
| [`blink/`](blink/) | Nháy LED PB8, in số lần nháy ra console USART1. Ghi thẳng thanh ghi, 1,6 KB flash | Build được, chưa chạy trên kit |

`tools/mkimage.py` là bản sao từ ak-mcu-base, dùng để gắn header 256 B mà bootloader đòi.

## Cần cài

- Rust (`rustup`) và target Cortex-M3: `rustup target add thumbv7m-none-eabi`
- `arm-none-eabi-objcopy` (có sẵn trong toolchain ARM của PlatformIO hoặc STM32CubeIDE)
- Python 3 cho `tools/mkimage.py`
- STM32CubeProgrammer và đầu dò ST-Link để nạp

Build cả workspace: `cargo build --release` ở thư mục gốc; output nằm ở `target/thumbv7m-none-eabi/release/`.
Cách gắn header và nạp: xem README của từng thư mục.

## Ràng buộc chung khi chạy sau bootloader

- Bảng vector ở `0x08003100` (256 B đầu vùng app là header ảnh), căn 256 B → bảng ngắt phải tự khai đúng 45 ngắt
  của L151CB, bảng mặc định của `cortex-m-rt` (1 KB) không link được. Chi tiết trong [`blink/README.md`](blink/README.md).
- RAM bắt đầu từ `0x20000100`: đầu RAM là vùng `.noinit` bootloader dùng để truyền lệnh nhảy và lý do reset.
- App nhận chip ở trạng thái sau reset (MSI 2 MHz, watchdog tắt) và tự đặt xung nhịp.
