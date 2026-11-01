# ak-mcu-rust — firmware Rust trên AK MCU KIT

Firmware viết bằng Rust cho AK MCU KIT (STM32L151CB-A: 128 KB flash, 32 KB RAM, 32 MHz), chạy **sau
bootloader của [ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base)**: không đè bootloader, nên kit vẫn quay về
firmware C bất cứ lúc nào bằng cách nạp lại `app.img` của bản C.

Gồm một chuỗi bài cho người mới học Rust nhúng (mỗi bài một ý, mã ngắn đủ chiếu lên clip) và thư viện bo mạch `ak-kit`
dùng chung. Không thay thế ak-mcu-base: kernel AK, bootloader và OTA vẫn là bản C.

Mọi bài dưới đây đã chạy trên kit (30/10/2026).

**So sánh Rust và C trên chính kit này** (số đo thật, lỗi thật đã gặp, khi nào chọn gì): [docs/rust-va-c.md](docs/rust-va-c.md).

## Đang viết C, Rust cho thêm gì? Xem nhanh

| Ở C | Ở Rust | Xem ở đâu |
|---|---|---|
| Đọc vượt mảng: ra rác, chạy tiếp | dừng đúng dòng, OLED hiện `PANIC main.rs dong 34` | bài 5 [`an-toan/`](an-toan/) |
| Task gửi message dạng byte, bên nhận tự ép kiểu | channel có kiểu: gửi nhầm kiểu là không build được | bài 4 [`dong-ho/`](dong-ho/) |
| Mã trả về dễ quên kiểm | `Result` buộc xử lý nhánh lỗi (RTC mất giờ thật trên kit) | bài 4, [`ak-kit/src/dong_ho.rs`](ak-kit/src/dong_ho.rs) |
| Việc nhiều bước chia thành task + signal + timer | `async`: viết tuần tự, `.await` là chỗ nhường lượt | bài 1–3 |
| Test hàng nghìn đầu vào ngẫu nhiên: tự dựng | `proptest`: 7 tính chất (byte rác, mất điện ở byte bất kỳ), tự thu nhỏ ví dụ hỏng | [`ak-boot/tests/tinh_chat.rs`](ak-boot/tests/tinh_chat.rs) |
| Không phải viết lại: | module Rust link vào firmware C, +312 B flash, đang chạy OTA trên kit | [`ak-ffi/`](ak-ffi/) |

Cái Rust **không** cứu được (bảng vector, I2C giả lập, ST-Link mất kết nối) cũng ghi đủ ở
[docs/rust-va-c.md](docs/rust-va-c.md#4-rust-không-cứu-được-gì--các-lỗi-thật-đã-gặp).

## Các bài

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
bằng ngắt EXTI, đồng hồ RTC, `in_ra!`, panic in tệp:dòng ra cổng COM và lên OLED, nghe lệnh nạp trên cổng COM).
Thiết kế chuỗi bài: [`docs/superpowers/specs/`](docs/superpowers/specs/).

## Nạp

```bash
python tools/nap.py <tên-bài> --com <cổng>  # qua cổng COM ảo của ST-Link — không cần nối SWD
python tools/nap.py <tên-bài>               # qua ST-Link (SWD)
```

Qua cổng COM: chương trình đang chạy (bài Rust nào cũng được, app C của ak-mcu-base cũng được) nhận lệnh LOADER,
kit vào bootloader, `tools/ak_fw.py` gửi ảnh, bootloader cài. Ảnh mới chạy thử: sau 30 s bài Rust tự xác nhận với
bootloader; nếu nó hỏng trước đó, sau 3 lần khởi động bootloader cài lại ảnh trước. Bài 2 (32 KB): ~9 s cả quy trình.
Lần đầu chuyển từ bài Rust cũ (trước khi có phần nghe lệnh nạp) thì phải dùng ST-Link một lần.

**Còi mặc định tắt** cho yên khi làm việc cạnh kit. Bài có tiếng (2, 3, 6) muốn nghe — ví dụ khi quay clip — thì thêm
`kit.coi.dat_am_luong(50);` ngay sau `ak_kit::init(spawner)` (1 = nhỏ nhất, 50 = to nhất).

`tools/mkimage.py` và `tools/ak_fw.py` là bản sao từ ak-mcu-base (gắn header ảnh, giao thức nạp `fw_proto`).
`ak-boot/` là phần logic để bài Rust sống chung bootloader (`boot_ctrl` trong EEPROM, khung `fw_proto`, CRC), không
phụ thuộc chip: `cargo test -p ak-boot --target <target của máy>` (14 test ví dụ, có đọc EEPROM thật dump từ kit,
và 7 test tính chất).

Bước tiếp theo (trộn Rust vào firmware C): [`docs/lo-trinh.md`](docs/lo-trinh.md).

## Cần cài

- Rust (`rustup`). Phiên bản Rust và target Cortex-M3 ghim trong [`rust-toolchain.toml`](rust-toolchain.toml); lần
  đầu chạy `cargo` trong repo, `rustup` tự tải đúng bản
- `arm-none-eabi-objcopy` (có sẵn trong toolchain ARM của PlatformIO hoặc STM32CubeIDE)
- Python 3 (+ `pyserial`) cho `tools/mkimage.py`, `tools/ak_fw.py`
- STM32CubeProgrammer và đầu dò ST-Link để nạp lần đầu

Build cả workspace: `cargo build --release` ở thư mục gốc; output nằm ở `target/thumbv7m-none-eabi/release/`.
`python tools/kich_thuoc.py --kiem` in flash/RAM từng bài và kiểm các ràng buộc ở mục dưới. CI
([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) chạy build, test, `clippy` và bước kiểm này mỗi lần push.
Cách gắn header và nạp: xem README của từng thư mục.

## Ràng buộc chung khi chạy sau bootloader

- Bảng vector ở `0x08003100` (256 B đầu vùng app là header ảnh), căn 256 B → bảng ngắt phải tự khai đúng 45 ngắt
  của L151CB, bảng mặc định của `cortex-m-rt` (1 KB) không link được. Chi tiết trong [`blink/README.md`](blink/README.md).
- RAM bắt đầu từ `0x20000100`: đầu RAM là vùng `.noinit` bootloader dùng để truyền lệnh nhảy và lý do reset.
- App nhận chip ở trạng thái sau reset (MSI 2 MHz, watchdog tắt) và tự đặt xung nhịp.
