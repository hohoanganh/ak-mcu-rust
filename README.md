# ak-mcu-rust — firmware Rust trên AK MCU KIT

<p align="center">
  <a href="https://github.com/hohoanganh/ak-mcu-rust/actions/workflows/ci.yml"><img src="https://github.com/hohoanganh/ak-mcu-rust/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/hohoanganh/ak-mcu-rust/commits/main"><img src="https://img.shields.io/github/last-commit/hohoanganh/ak-mcu-rust?label=commit%20cu%E1%BB%91i" alt="last commit"></a>
  <a href="https://github.com/hohoanganh/ak-mcu-rust/blob/main/rust-toolchain.toml"><img src="https://img.shields.io/badge/Rust-1.97%20%C2%B7%20no__std-000000?logo=rust" alt="Rust 1.97 no_std"></a>
  <a href="https://embassy.dev"><img src="https://img.shields.io/badge/Embassy-async-e36209" alt="Embassy async"></a>
  <a href="https://github.com/hohoanganh/ak-mcu-base"><img src="https://img.shields.io/badge/chip-STM32L151-03234B?logo=stmicroelectronics" alt="chip STM32L151"></a>
  <br>
  <a href="https://github.com/hohoanganh/ak-mcu-rust/tree/main/bai"><img src="https://img.shields.io/badge/b%C3%A0i-7-2ea44f" alt="7 bài"></a>
  <a href="https://github.com/hohoanganh/ak-mcu-rust/blob/main/Cargo.toml"><img src="https://img.shields.io/badge/crate-5-6f42c1" alt="5 crate"></a>
  <a href="https://github.com/hohoanganh/ak-mcu-base"><img src="https://img.shields.io/badge/ch%E1%BA%A1y%20sau-bootloader%20C%20c%E1%BB%A7a%20ak--mcu--base-0366d6" alt="chạy sau bootloader của ak-mcu-base"></a>
  <a href="https://claude.com/claude-code"><img src="https://img.shields.io/badge/built%20with-Claude%20Code-d97757?logo=claude&amp;logoColor=white" alt="built with Claude Code"></a>
</p>

Chuỗi bài Rust nhúng cho người đang viết C, chạy trên AK MCU KIT (STM32L151, 128 KB flash, 32 KB RAM). Bài Rust chạy
**sau bootloader C** của [ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base), nên kit luôn quay về firmware C được.
Mọi bài đã chạy trên kit thật.

## Rust cho thêm gì so với C?

| Ở C | Ở Rust | Xem |
|---|---|---|
| Đọc vượt mảng: ra rác, chạy tiếp | dừng đúng dòng, OLED hiện `PANIC main.rs dong 34` | bài 5 |
| Message là byte, bên nhận tự ép kiểu | channel có kiểu: gửi nhầm là không build được | bài 4 |
| Mã trả về dễ quên kiểm | `Result` buộc xử lý lỗi | bài 4 |
| Việc nhiều bước chia thành task + signal + timer | `async`: viết tuần tự từ trên xuống | bài 1–3 |
| Test hàng nghìn đầu vào ngẫu nhiên phải tự dựng | `proptest` có sẵn, tự thu nhỏ ví dụ hỏng | [`ak-boot`](ak-boot/tests/tinh_chat.rs) |

Không cần viết lại firmware C: hai module Rust chạy bên trong nó qua [`ak-ffi`](ak-ffi/). Bộ tách khung nạp firmware
chỉ tốn thêm 312 B. Bộ tách yêu cầu Modbus ([`ak-modbus`](ak-modbus/)) thay thư viện C nanoMODBUS, firmware nhỏ đi 3,4 KB,
và trả lời giống thư viện đó từng byte trên 20 000 yêu cầu ngẫu nhiên.

## Các bài

Tất cả bài nằm trong [`bai/`](bai/). Cột bên phải là màn hình OLED 128×64 của kit khi bài chạy.

| Bài | Nội dung | Màn hình OLED |
|---|---|---|
| 0 [`blink`](bai/blink/) | Nháy LED bằng cách ghi thẳng thanh ghi, không thư viện | không dùng màn hình |
| 1 [`hello-async`](bai/hello-async/) | Chờ mà không chặn: LED nháy và đếm giây cùng lúc | không dùng màn hình |
| 2 [`ba-viec-cung-luc`](bai/ba-viec-cung-luc/) | Ba việc ba nhịp: LED, còi, đồng hồ trên OLED | <img src="docs/anh/bai2-ba-viec-cung-luc.gif" width="260" alt="Bài 2: đồng hồ đếm giây, LED 300 ms, còi 1 s"> |
| 3 [`nut-bam`](bai/nut-bam/) | Chờ sự kiện: đàn ba phím | <img src="docs/anh/bai3-nut-bam.gif" width="260" alt="Bài 3: bấm nút hiện nốt DO, MI, SOL và số lần bấm"> |
| 4 [`dong-ho`](bai/dong-ho/) | Hai task nói chuyện qua channel: đồng hồ kim từ RTC | <img src="docs/anh/bai4-dong-ho.gif" width="260" alt="Bài 4: đồng hồ kim và giờ số"> |
| 5 [`an-toan`](bai/an-toan/) | Đọc vượt mảng: Rust dừng và báo đúng dòng | <img src="docs/anh/bai5-an-toan.gif" width="260" alt="Bài 5: đọc diem[5] ngoài mảng, màn hình PANIC main.rs dòng 34"> |
| 6 [`cua-ferris`](bai/cua-ferris/) | Trò chơi Ferris hứng sao, 30 khung/giây | <img src="docs/anh/bai6-cua-ferris.gif" width="260" alt="Bài 6: Ferris chạy hứng sao rơi"> |

Phần cứng của kit (LED, còi, OLED, nút, RTC, cổng COM) gói sẵn trong [`ak-kit`](ak-kit/src/lib.rs), nên mỗi bài chỉ
còn đúng ý cần dạy. Phần vẽ màn hình tách riêng trong [`ak-hinh`](ak-hinh/) (không phụ thuộc chip): cùng mã đó chạy trên
kit và chạy trên máy tính để dựng GIF, nên hình trong bảng là hình bài thật sự vẽ ra.

## Chạy thử

Cần: [Rust](https://rustup.rs) (phiên bản ghim sẵn, tự tải), `arm-none-eabi-objcopy` (có trong PlatformIO hoặc
STM32CubeIDE), Python 3 + `pyserial`.

```bash
python tools/nap.py hello-async --com COM3   # build và nạp qua cổng COM, không cần ST-Link
python tools/nap.py hello-async              # hoặc nạp qua ST-Link
```

Kit đang chạy firmware C hay bài Rust nào cũng nạp qua cổng COM được. Bài Rust bị kẹt thì giữ nút giữa và bấm
RESET để vào chế độ nạp, rồi nạp bằng ST-Link.

## Đọc thêm

- [So sánh Rust và C](docs/rust-va-c.md): số đo thật, lỗi thật đã gặp, cái Rust không cứu được, khi nào chọn gì.
- [Sơ đồ luồng chạy](docs/so-do.md): bộ nhớ, khởi động, async, nạp và quay lui, Rust trong firmware C.
- [Lộ trình](docs/lo-trinh.md): đã làm gì, sắp làm gì.
- [Bẫy kỹ thuật](CLAUDE.md): bảng vector 256 B, RAM, I2C, ST-Link…

Dựng lại GIF sau khi sửa phần vẽ (cần Python + `pillow`):

```bash
cargo run --release -p ak-hinh --example quay --target x86_64-pc-windows-gnu -- target/quay
python tools/demo_gif.py target/quay docs/anh
```
