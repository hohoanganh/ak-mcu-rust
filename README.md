# ak-mcu-rust — firmware Rust trên AK MCU KIT

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

Không cần viết lại firmware C: một module Rust đã chạy bên trong nó, chỉ tốn thêm 312 B ([`ak-ffi`](ak-ffi/)).

## Các bài

| Bài | Nội dung |
|---|---|
| 0 [`blink`](blink/) | Nháy LED bằng cách ghi thẳng thanh ghi, không thư viện |
| 1 [`hello-async`](hello-async/) | Chờ mà không chặn: LED nháy và đếm giây cùng lúc |
| 2 [`ba-viec-cung-luc`](ba-viec-cung-luc/) | Ba việc ba nhịp: LED, còi, đồng hồ trên OLED |
| 3 [`nut-bam`](nut-bam/) | Chờ sự kiện: đàn ba phím |
| 4 [`dong-ho`](dong-ho/) | Hai task nói chuyện qua channel: đồng hồ kim từ RTC |
| 5 [`an-toan`](an-toan/) | Đọc vượt mảng: Rust dừng và báo đúng dòng |
| 6 [`cua-ferris`](cua-ferris/) | Trò chơi Ferris hứng sao, 30 khung/giây |

Phần cứng của kit (LED, còi, OLED, nút, RTC, cổng COM) gói sẵn trong [`ak-kit`](ak-kit/src/lib.rs), nên mỗi bài chỉ
còn đúng ý cần dạy.

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
