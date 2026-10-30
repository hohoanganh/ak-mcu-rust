# ak-mcu-rust — firmware Rust trên AK MCU KIT

Firmware viết bằng Rust cho AK MCU KIT (STM32L151CB-A: 128 KB flash, 32 KB RAM, 32 MHz), chạy **sau
bootloader của [ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base)**: không đè bootloader, nên kit vẫn quay về
firmware C bất cứ lúc nào bằng cách nạp lại `app.img` của bản C.

Gồm một chuỗi bài cho người mới học Rust nhúng (mỗi bài một ý, mã ngắn đủ chiếu lên clip) và thư viện bo mạch `ak-kit`
dùng chung. Không thay thế ak-mcu-base: kernel AK, bootloader và OTA vẫn là bản C.

Mọi bài dưới đây đã chạy trên kit (30/10/2026).

| Thư mục | Nội dung |
|---|---|
| [`blink/`](blink/) | Nháy LED PB8, in số lần nháy ra console USART1. Ghi thẳng thanh ghi, 1,6 KB flash |
| [`hello-async/`](hello-async/) | **Bài 1** — chờ mà không chặn: LED nháy và đếm giây cùng lúc bằng Embassy async |
| [`ba-viec-cung-luc/`](ba-viec-cung-luc/) | **Bài 2** — ba việc cùng lúc: LED, còi, đồng hồ trên OLED, ba nhịp khác nhau |
| [`nut-bam/`](nut-bam/) | **Bài 3** — chờ sự kiện: đàn ba phím, `select3` chờ cả ba nút trong một dòng |
| [`dong-ho/`](dong-ho/) | **Bài 4** — hai task nói chuyện qua channel: đọc RTC, vẽ đồng hồ kim trên OLED |
| [`an-toan/`](an-toan/) | **Bài 5** — đọc vượt mảng: Rust dừng và hiện PANIC kèm tệp, số dòng lên OLED |
| [`cua-ferris/`](cua-ferris/) | **Bài 6** — trò chơi Ferris hứng sao: 30 khung/giây, âm thanh qua channel |

`ak-kit/` là thư viện bo mạch dùng chung cho các bài (xung nhịp, LED, còi, OLED vẽ bằng `embedded-graphics`, ba nút
bằng ngắt EXTI, đồng hồ RTC, `in_ra!`, panic in tệp:dòng ra cổng COM và lên OLED). Nạp một bài:
`python tools/nap.py <tên-bài>`. Thiết kế chuỗi bài: [`docs/superpowers/specs/`](docs/superpowers/specs/).

**Còi mặc định tắt** cho yên khi làm việc cạnh kit. Bài có tiếng (2, 3, 6) muốn nghe — ví dụ khi quay clip — thì thêm
`kit.coi.dat_am_luong(50);` ngay sau `ak_kit::init()` (1 = nhỏ nhất, 50 = to nhất).

`tools/mkimage.py` là bản sao từ ak-mcu-base, dùng để gắn header 256 B mà bootloader đòi.

Các bước tiếp theo (OTA cho app Rust, trộn Rust vào firmware C): [`docs/lo-trinh.md`](docs/lo-trinh.md).

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
