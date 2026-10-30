# Lộ trình Rust trên AK MCU KIT

> Viết 30/10/2026, sau khi `blink/` build được (chưa chạy trên kit). Mỗi giai đoạn có **cổng quyết định**: chỉ đi
> tiếp khi giai đoạn trước có số đo trên kit thật, không đi tiếp vì "đã lên kế hoạch".

## Mục tiêu và thứ không làm

- **Mục tiêu:** biết chắc Rust làm được gì trên chip nhỏ này, tốn bao nhiêu flash/RAM so với C, và tìm con đường
  đưa Rust vào sản phẩm mà không phải viết lại ak-mcu-base.
- **Không làm:** viết lại kernel AK, bootloader, OTA bằng Rust. Bootloader C giữ nguyên; mọi app Rust chạy sau nó.

## Hiện trạng hệ sinh thái (đã tra 30/10/2026)

| | Tình trạng | Ý nghĩa cho kit |
|---|---|---|
| **Embassy** (`embassy-stm32`) | Có ví dụ chính thức cho STM32L1 trong repo embassy: `blinky`, `usart`, `spi`, `flash`, `eeprom`, `usb_serial`, dựng trên **`stm32l151cb-a`**; thư mục ví dụ còn được sửa ngày 19/10/2026 (driver flash hợp nhất) | Con đường chính. Có sẵn driver EEPROM — đúng chỗ bootloader để `boot_ctrl` |
| Dữ liệu chip của embassy | `STM32L151CB` và `STM32L151CB-A` đều **45 ngắt** → bảng vector 244 B, vừa địa chỉ `0x08003100` | Bẫy bảng vector 1 KB của `blink/` không lặp lại với Embassy (cần kiểm bằng build) |
| | `STM32L151CB` có 16 KB RAM, **`CB-A` có 32 KB** | Kit đọc ra device ID `0x429` (dòng Cat.2, "…BxxA") → nhiều khả năng là CB-A. Nếu đúng, RAM gấp đôi cho cả C lẫn Rust |
| `stm32l1xx-hal` | Có, hỗ trợ L100/L151/L152/L162, ít được bảo trì hơn Embassy | Phương án dự phòng |
| PAC `stm32l1` | Có (svd2rust); bảng ngắt khai theo dòng lớn, không link được ở `0x08003100` | Không dùng trực tiếp làm `rt` |
| **defmt + RTT** qua probe-rs | RTT đọc RAM qua SWD, **không cần chân SWO** (ST-Link tự thiết kế không có SWO) | Log nhanh, gọn hơn `printf` qua UART. Chưa kiểm probe-rs với đầu dò này |
| RTIC 2 | Lập lịch bằng ngắt phần cứng, một stack chung, không cần cấp phát động | Gần mô hình ưu tiên của kernel AK nhất. Chưa có số đo RAM trên Cortex-M3 |

## Các giai đoạn

### 0. Nháy LED bằng thanh ghi — **xong (30/10/2026)**
`blink/`: 1 604 B flash, chạy trên kit sau bootloader 1.4.0. Kit là STM32L151CB-A, **32 KB RAM** (đã kiểm), nên mục
"xác định RAM thật" của giai đoạn 1 đã xong.

### 1. Chạy được và nhìn được (½ ngày, cần kit)
1. Nạp `blink.img`, xác nhận LED và dòng `nhay lan N` trên COM.
2. **Xác định RAM thật:** đọc chữ trên chip; thử ghi/đọc `0x20004000` qua SWD. Kết quả ghi vào `CLAUDE.md` của cả hai
   repo (ak-mcu-base đang khai 16 KB).
3. Cài `probe-rs`, thử `probe-rs attach --chip STM32L151CBxxA` với đầu dò ST-Link V2-1. Được thì dùng defmt/RTT cho các
   giai đoạn sau; không được (đầu dò, `mode=UR`) thì giữ log UART.

**Cổng:** LED chạy, biết RAM 16 hay 32 KB, biết probe-rs dùng được hay không.

### 2. Chuyển sang Embassy (1–2 ngày)
1. `blink-embassy/`: `embassy-stm32` (feature đúng chip, **bỏ feature `memory-x`** để giữ `memory.x` riêng ở
   `0x08003100` / `0x20000100`), `embassy-executor`, `embassy-time`.
2. Console USART1 kiểu async: echo dòng lệnh, lệnh `ver`.
3. **Đo:** flash, RAM tĩnh, mức stack, so với `blink/` và với ak-mcu-base (kernel + shell).

**Cổng:** bảng vector ≤ 256 B (kiểm bằng `arm-none-eabi-nm`), app Embassy chạy sau bootloader, tốn RAM bao nhiêu.
RAM quá 50 % trên chip 16 KB thì cân nhắc dừng Embassy ở mức thử nghiệm.

### 3. Làm "công dân" của bootloader (2–3 ngày) — giai đoạn quan trọng nhất
Không có bước này thì mỗi lần cập nhật app Rust phải cắm ST-Link.
1. Crate `ak-boot` (`no_std`, không phụ thuộc chip): đọc/ghi `boot_ctrl` (hai bản ghi 32 B luân phiên ở EEPROM
   `0x08080000`), `confirm()` cho ảnh đang chạy thử. Chuyển định dạng từ `boot_ctrl.c` của ak-mcu-base.
2. Con tối thiểu của `fw_proto` trên console: `INFO`, `LOADER`, `RESET`, `CONFIRM`. Lệnh `LOADER` reset vào bootloader,
   phần nạp còn lại để bootloader C làm → `tools/ak_fw.py flash` của ak-mcu-base dùng được cho app Rust.
3. **Test trên máy tính bằng `cargo test`**: crate `ak-boot` dựng trên PC, chạy cùng bộ vector thử với
   `tests/test_fw.c` của bản C (mất điện giữa lúc ghi bản ghi, CRC sai, cờ thử). Đây là nơi Rust có lợi rõ nhất.

**Cổng:** cập nhật app Rust bằng `ak_fw.py` qua COM, không cần ST-Link; ảnh cài qua OTA tự xác nhận, không bị
bootloader quay lui.

### 4. Ngoại vi của kit (mỗi món ½–1 ngày, chọn theo nhu cầu)
- OLED + `embedded-graphics`: một màn hình đồng hồ, đo thời gian một khung so với bản C (C: Tetris 3,8 ms/khung).
- SHT45 qua I2C (driver có sẵn trên crates.io).
- Nút bấm bằng ngắt EXTI + `embassy-sync` channel — tương đương "nút thành message" của kernel AK.
- RS485 Modbus slave: thử crate có sẵn; đối chiếu với `tools/ak_mb.py` (đã có bộ thử 841 cặp ghi/đọc cho bản C).

**Cổng:** có một chương trình Rust làm được việc thật trên kit, kèm bảng so flash/RAM/thời gian với bản C.

### 5. Trộn Rust vào firmware C (2–3 ngày) — con đường thực tế cho sản phẩm
Không viết lại gì: biên dịch một module Rust thành thư viện tĩnh (`crate-type = ["staticlib"]`), link vào ak-mcu-base,
gọi qua `extern "C"`.
- Ứng viên đầu tiên: **bộ phân tích khung** (khung Modbus hoặc `fw_proto`) — nhận byte từ bên ngoài, chỗ hay có lỗi
  tràn bộ đệm nhất, lại có sẵn test để so.
- Việc phải giải: thêm bước build Rust vào PlatformIO (`extra_scripts`) và CMake; `panic` của Rust gọi `FATAL()` của
  kernel AK để vào nhật ký sự cố; kiểm flash tăng bao nhiêu.

**Cổng:** bản C có module Rust qua đủ `make test` + build PlatformIO, chạy trên kit. Từ đây quyết định có đưa Rust vào
dự án thật nào không.

### 6. Tuỳ chọn — mô hình AK bằng Rust
Chỉ làm nếu giai đoạn 2–5 cho thấy đáng: viết lại mô hình task + message + ưu tiên bằng Rust (hàng đợi `heapless`),
hoặc dùng RTIC 2 vì nó lập lịch theo ưu tiên ngắt giống kernel AK. Đây là việc lớn, không cần cho mục tiêu học.

## Số cần đo ở mỗi giai đoạn

| Số đo | Cách lấy |
|---|---|
| Flash, RAM tĩnh | `arm-none-eabi-size` trên file ELF |
| Mức stack cao nhất | tô mẫu vùng stack lúc khởi động, đọc lại (như lệnh `stat` của bản C) |
| Thời gian một việc (khung OLED, một yêu cầu Modbus) | chân GPIO bật/tắt + máy hiện sóng, hoặc đếm `DWT->CYCCNT` |
| Thời gian build sạch | `cargo build --release` sau `cargo clean` |

## Rủi ro

- **Embassy đổi API nhanh** (ví dụ L1 dùng edition 2024, executor 0.10): ghim phiên bản trong `Cargo.lock`, chỉ nâng có chủ đích.
- **Đầu dò ST-Link tự thiết kế** có thể không hợp với probe-rs; vẫn còn STM32_Programmer_CLI + UART.
- **RAM 16 KB** nếu chip là CB thường: Embassy + defmt có thể chật; RTIC hoặc ghi thẳng thanh ghi là đường lui.
- Một người bảo trì hai ngôn ngữ: giữ Rust ở mức thử nghiệm cho tới khi giai đoạn 5 qua cổng.

## Nguồn

- Ví dụ STM32L1 của Embassy: <https://github.com/embassy-rs/embassy/tree/main/examples/stm32l1>
- Tài liệu `embassy-stm32` cho `stm32l151cb`: <https://docs.embassy.dev/embassy-stm32/git/stm32l151cb/index.html>
- Dữ liệu chip (số ngắt, bộ nhớ): <https://github.com/embassy-rs/stm32-data-generated> (`data/chips/STM32L151CB.json`, `STM32L151CB-A.json`)
- Tạo dự án Embassy mới: <https://embassy.dev/book/dev/new_project.html>
- `stm32l1xx-hal`: <https://docs.rs/crate/stm32l1xx-hal/0.1.0/>
- PAC `stm32l1`: <https://docs.rs/crate/stm32l1/latest>
- RTT không cần SWO: <https://forum.segger.com/index.php/Thread/3281-SOLVED-ST-Nucleo-F411-SWO>
- defmt qua RTT: <https://docs.rs/crate/defmt-rtt/1.2.0/source/src/lib.rs>
- RTIC: <https://docs.rs/crate/cortex-m-rtic/^1.0>
- Thông số STM32L151CB: <https://www.st.com/en/microcontrollers-microprocessors/stm32l151cb.html>
