# Thiết kế: chuỗi demo Rust cho người mới trên AK MCU KIT

> 30/10/2026. Đã thống nhất với Hoàng Anh: đối tượng là sinh viên / người mới (clip TikTok); **mỗi demo một chương
> trình riêng**; nền tảng **Embassy (async)**; danh sách 7 bài bên dưới.

## Mục tiêu

Mỗi bài là một clip 30–60 s, minh họa **đúng một ý**, mã `main.rs` đủ ngắn (≈ 50–100 dòng) để chiếu lên clip và đọc
từ trên xuống là hiểu. Người xem mang về: Rust nhúng dễ đọc, làm nhiều việc cùng lúc không cần RTOS, và an toàn hơn C.

Không làm trong chuỗi này: OTA cho app Rust (giai đoạn 3 của `docs/lo-trinh.md`), bản menu gộp các bài, so sánh tốc
độ với C.

## Danh sách bài

| Bài | Thư mục | Câu chốt | Trên kit |
|---|---|---|---|
| 0 | `blink` (đã có, giữ nguyên) | Rust chạm tới tận thanh ghi | LED nháy |
| 1 | `hello-async` | Chờ mà không chặn | LED nháy + dòng chào trên cổng COM |
| 2 | `ba-viec-cung-luc` | 3 việc cùng lúc, không cần RTOS | LED, còi, bộ đếm OLED chạy ba nhịp khác nhau |
| 3 | `nut-bam` | Chờ sự kiện, CPU được nghỉ | 3 nút → LED / còi / OLED phản hồi tức thì |
| 4 | `cam-bien` | Các task nói chuyện qua channel (như message của kernel AK) | SHT45: số + đồ thị nhiệt độ, độ ẩm trên OLED |
| 5 | `an-toan` | Lỗi C âm thầm hỏng bộ nhớ, Rust dừng và chỉ đúng dòng | Bấm nút đọc vượt mảng → OLED hiện `panic` kèm tên file, số dòng |
| 6 | `cua-ferris` | Rust vẽ mượt trên chip 32 MHz | Chú cua Ferris (hình pixel tự vẽ) chạy trên OLED, nút điều khiển, còi kêu |

## Kiến trúc

```
ak-mcu-rust/
├── Cargo.toml            workspace: thêm ak-kit và 6 bài mới
├── memory.x              dùng chung cho các bài Embassy (xem "Bộ nhớ")
├── ak-kit/               thư viện bo mạch — người xem KHÔNG cần đọc
│   └── src/lib.rs        init(), Kit { led, coi, nut1..3, oled, console }, panic handler
├── hello-async/ … cua-ferris/   mỗi bài: Cargo.toml + src/main.rs + README.md
└── tools/
    ├── mkimage.py        (đã có)
    └── nap.py            build → .bin → .img → nạp qua ST-Link, một lệnh
```

**`ak-kit`** gói mọi thứ khởi tạo phần cứng để `main.rs` của mỗi bài chỉ còn đúng ý cần dạy:

| Thành phần | Phần cứng | Cách làm |
|---|---|---|
| Xung nhịp | 32 MHz (PLL từ HSI) như bản C | `embassy_stm32::Config` |
| LED | PB8, sáng khi mức thấp | `Output`, bọc thành `Led::bat()/tat()/dao()` để khỏi lộ chuyện mức thấp |
| Còi | PB0 = TIM3_CH3, còi thụ động | `SimplePwm`; `Coi::keu(tan_so_hz, ms).await` |
| Nút 1–3 | PB3, PC13, PB4, nối GND, kéo lên trong | `ExtiInput`; `nut.cho_bam().await` |
| OLED | SSD1309 128×64, I2C 0x3C trên PB13 (SCL) / PB12 (SDA), RES PA15 | **I2C giả lập bằng GPIO** (hai chân này không thuộc khối I2C phần cứng) + crate `ssd1306` chế độ bộ đệm khung + `embedded-graphics` |
| Cảm biến | SHT45 0x44 trên I2C1 PB6/PB7 | I2C phần cứng của Embassy, đọc lệnh `0xFD` (đo chính xác cao, 8,3 ms) + kiểm CRC |
| Console | USART1 PA9/PA10, 115200 | `println!`-kiểu đơn giản qua `core::fmt::Write` |
| Panic | — | Handler trong `ak-kit`: in `panic: <file>:<dòng>` ra console **và** lên OLED, rồi nháy LED; bài 5 dựa vào đây |

Tên hàm trong `ak-kit` dùng tiếng Việt không dấu (`cho_bam`, `keu`) để người mới đọc clip hiểu ngay; tên của Rust và
Embassy giữ nguyên.

## Bộ nhớ và bootloader

- Bootloader trên kit: ak-mcu-base 1.4.0 build 32K. `memory.x` cho các bài Embassy:
  `FLASH 0x08003100, 116K − 256` · `RAM 0x20000100, 32K − 256`.
- Bảng vector: `embassy-stm32` với feature chip `stm32l151cb-a` (đúng chip của kit, Cat.2) — theo dữ liệu chip, 45 ngắt
  → bảng 244 B, căn 256 B được. **Kiểm bằng build ở bài 1**; nếu sai thì dừng, không đi tiếp.
- Không bật feature `memory-x` của `embassy-stm32` (nó sinh `memory.x` ở `0x08000000`, đè bootloader).
- Bài 0 (`blink`) giữ `memory.x` riêng 16K như hiện tại.

## Bộ chạy task

**`platform-spin`, không phải `platform-cortex-m`** (đổi sau khi thử trên kit, 30/10/2026): bộ chạy mặc định ngủ bằng WFE
và lúc chip ngủ thì ST-Link của kit (không có NRST) không vào được để nạp lại. Mỗi bài viết
`#[embassy_executor::main(entry = "cortex_m_rt::entry")]`. Kit cắm USB nên tốn điện hơn không thành vấn đề.

## Nạp

Các bài Rust chưa nói được giao thức `fw_proto`, nên không nạp qua cổng COM được. Nạp bằng ST-Link vào `0x08003000`
(bootloader giữ nguyên). `tools/nap.py <bai>` làm trọn: `cargo build --release -p <bai>` → `arm-none-eabi-objcopy` →
`mkimage.py build --load-addr 0x08003100 --board ak-l151` → `STM32_Programmer_CLI -c port=SWD mode=UR -d … -v -rst`.
Quay về firmware C: nạp lại `app.img` của bản C.

## README mỗi bài (phục vụ quay clip)

1. Câu chốt (một dòng).
2. Lệnh nạp.
3. Quay gì: góc máy, bấm nút nào, thời lượng.
4. Những dòng code nên khoanh lại trên clip (số dòng).
5. Kích thước: flash, RAM (từ `arm-none-eabi-size`).

## Kiểm thử

- Mỗi bài: build không cảnh báo; kiểm bảng vector ≤ 256 B (`arm-none-eabi-nm`, địa chỉ `__INTERRUPTS`/`__EXCEPTIONS`).
- Trên kit: mỗi bài in trạng thái ra console (số đếm, nút vừa bấm, số đo) để kiểm được từ máy tính; phần nhìn (OLED,
  LED) và nghe (còi) nhờ Hoàng Anh xác nhận.
- Bài 4: số đo SHT45 so với lệnh `th` của firmware C (cùng kit, cùng phòng), lệch ≤ 0,5 °C.
- Bài 5: panic phải hiện đúng tên file và số dòng của chỗ đọc vượt mảng.

## Thứ tự làm và điểm dừng

1. `ak-kit` (LED, console, panic) + bài 1 → **dừng nếu Embassy không chạy được sau bootloader.**
2. OLED (I2C giả lập + `ssd1306`) + bài 2 → **dừng nếu SSD1309 không lên hình với `ssd1306`** (khi đó viết driver nhỏ
   dựa trên trình tự khởi tạo của `kit.c`).
3. Còi, nút → bài 3. 4. SHT45 → bài 4. 5. Bài 5. 6. Bài 6.

## Rủi ro

| Rủi ro | Cách xử lý |
|---|---|
| Phiên bản Embassy trên crates.io khác ví dụ trong repo embassy | Ghim phiên bản trong `Cargo.lock`; theo ví dụ `examples/stm32l1` của đúng phiên bản đó |
| PB3 sau reset là chân JTAG (JTDO) | Embassy cấu hình lại khi tạo `ExtiInput`; kiểm nút 1 ở bài 3 |
| RAM: `ssd1306` bộ đệm khung 1 KB + executor + task | Đo ở từng bài; còn 32K nên dư |
| Hình Ferris | Tự vẽ pixel art 1 bit (linh vật Ferris thuộc phạm vi công cộng — CC0), không chép ảnh có sẵn |
