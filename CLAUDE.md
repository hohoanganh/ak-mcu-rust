# CLAUDE.md

Ghi chú cho Claude khi làm việc trong repo này. Trả lời và viết tài liệu bằng tiếng Việt.

## Repo này là gì

Firmware Rust trên AK MCU KIT (STM32L151), **chạy sau bootloader của
[ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base)**. Không thay ak-mcu-base: kit phải luôn quay về được firmware
C bằng cách nạp lại `app.img` của bản C. Lộ trình: `docs/lo-trinh.md`.

Workspace Cargo: mỗi thử nghiệm một thư mục (`blink/`…), `profile.release` khai ở `Cargo.toml` gốc, output ở `target/`.

## Bẫy đã gặp — đọc trước khi sửa `memory.x`, bảng vector hay thêm crate

- **Bảng vector chỉ được ≤ 256 B.** Bootloader đặt `VTOR` ở `0x08003100` (256 B đầu vùng app là header ảnh), địa chỉ
  này chỉ căn 256 B. `cortex-m-rt` mặc định sinh 240 ngắt (bảng 1 KB, đòi căn 1 KB) → lỗi link
  `ERROR(cortex-m-rt): Interrupt vector table misalignment detected`. Không có HAL/PAC thì bật feature `device` của
  `cortex-m-rt` và tự khai `__INTERRUPTS: [usize; 45]` (`blink/src/main.rs`, `device.x` để trống).
- **PAC `stm32l1` (feature `stm32l151`) không cứu được:** nó khai bảng ngắt theo dòng dung lượng lớn, bảng vẫn vượt
  256 B. Theo dữ liệu chip của embassy (`stm32-data-generated`), cả `STM32L151CB` lẫn `STM32L151CB-A` có đúng
  **45 ngắt** → `embassy-stm32` với feature `stm32l151cb-a` ra bảng 244 B (đã kiểm: mọi bài Embassy chạy ở `0x08003100`).
- **RAM bắt đầu `0x20000100`.** Đầu RAM là `.noinit` của bootloader (52 B: lệnh nhảy, lý do reset, bản ghi sự cố);
  bootloader đọc nó sau mỗi reset. Đừng để `.data`/`.bss`/stack của Rust đè lên.
- **Ảnh phải có header 256 B:** `tools/mkimage.py build <bin> --load-addr 0x08003100 --board ak-l151`. Ảnh trần hay sai
  tên board thì bootloader không chạy. `tools/mkimage.py` là bản sao từ ak-mcu-base (commit ghi ở đầu file): bên đó
  đổi định dạng header thì chép lại.
- **App nhận chip ở trạng thái sau reset** (bootloader nhảy qua một lần reset): MSI 2 MHz, watchdog tắt, `VTOR` đã
  đặt. App tự đặt xung nhịp; lên 16 MHz ở dải điện áp mặc định (range 2) phải bật `ACC64` rồi mới đặt 1 wait state.
- **Ảnh cài qua OTA chạy "thử"**: app phải tự xác nhận (ghi `boot_ctrl` trong EEPROM, như `boot_ctrl_confirm()` của
  bản C), không thì sau 3 lần khởi động bootloader cài lại ảnh cũ. Nạp qua SWD thì không bị (không có cờ thử).
- **Device ID đọc qua SWD là `0x429`** (dòng Cat.2 — "STM32L15xxBxxA"), theo dữ liệu chip thì L151CB-A có **32 KB
  RAM** — đã kiểm trên kit 30/10/2026 (ghi/đọc `0x20007FF0` qua SWD, không lặp địa chỉ; bản C 32K chạy). ak-mcu-base có
  `ram_size_kb`, **mặc định 32** từ commit `9d23312` (16K cũ sót lại từ kit 2.1). Muốn nới `memory.x` lên 32 KB thì kit
  phải mang bootloader build 32K (bản mặc định từ `9d23312`). Bootloader chỉ nhận stack
  pointer ban đầu của app ≤ RAM nó được build với — bootloader 16K từ chối app Rust có stack ở `0x20008000`.

- **Chip ngủ bằng WFE thì ST-Link không vào được** (`Unable to get core ID`, `DEV_TARGET_CMD_ERR`, có lúc làm đầu dò treo
  tới mức phải rút USB). Bộ chạy task `platform-cortex-m` của Embassy ngủ WFE tới khi có việc; cờ `DBG_SLEEP` không cứu
  được, ngắt SysTick 1 ms đánh thức cũng không đủ. Đầu dò không có NRST nên không kéo chip ra được. **Các bài dùng
  `platform-spin`** + `#[embassy_executor::main(entry = "cortex_m_rt::entry")]`: không ngủ, ST-Link luôn vào được, nạp
  chồng lên bài đang chạy bình thường (30/10/2026). `ak-kit` không phụ thuộc `embassy-executor` để hai kiểu không đụng nhau.
- **Kẹt rồi thì cứu:** giữ **nút giữa (PC13)** khi bấm RESET → `ak_kit::init()` dừng ở chế độ nạp (không chạy Embassy).
  Chương trình không gọi `ak_kit::init()` thì dùng **BOOT0**: giữ BOOT0, bấm RESET → bootloader ROM, SWD vào được.
- **I2C giả lập của OLED (PB13/PB12) phải nghỉ sau khi đổi SDA, trước khi kéo SCL lên.** Dây SDA của màn chỉ có điện
  trở kéo lên yếu, lên mức chậm; ở 32 MHz kéo SCL ngay lệnh kế tiếp thì màn đọc sai bit, không trả ACK ở bất kỳ địa chỉ
  nào. Ở xung nhịp thấp (MSI) lỗi không lộ ra vì CPU tự chậm; RTC trên PB6/PB7 (kéo lên mạnh) cũng không lộ. Nghỉ sau
  sườn SCL không cứu được — chỗ thiếu là giữa SDA và SCL (30/10/2026, mất ~1 giờ: đã thử tốc độ chân, đảo chân, reset
  dài, ghi thẳng thanh ghi; mấu chốt là thử `embassy_stm32::init` với cấu hình mặc định thì màn trả lời).
- **Còi TIM3, đồng hồ Embassy TIM2** (`time-driver-tim2`): `time-driver-any` có thể lấy trúng TIM3.
- Embassy 0.6.0 (crates.io) đặt tên enum theo PAC: `PllSource::HSI`, `PllMul::MUL6`, `PllDiv::DIV3`, `Sysclk::PLL1_R`
  (ví dụ trên nhánh chính của repo embassy viết `Hsi`, `Mul6`…). Macro `main` cần `cortex-m-rt` là phụ thuộc trực tiếp.

- **USART của L151 chỉ giữ 1 byte nhận.** Task hỏi thanh ghi RX sẽ mất byte khi task khác bận (vẽ OLED tới 25 ms).
  `ak-kit/src/nap.rs` nhận bằng ngắt RXNE (handler riêng qua `bind_interrupts!`) vào hàng đợi 256 B; phía gửi vẫn chặn,
  vì `in_ra!` ghi trong critical section và panic chạy khi đã tắt ngắt — `BufferedUart` (gửi bằng ngắt) sẽ treo ở đó.
- `ak_kit::init(spawner)` tự chạy task nghe lệnh nạp; INFO trả `staging 0` — bài Rust không nhận ảnh khi đang chạy,
  phải `ak_fw.py loader` trước (`nap.py --com` làm sẵn). Flash dùng kiểu `flash::Blocking`, khác `mode::Blocking`.

## Build, nạp

- Phiên bản Rust ghim trong `rust-toolchain.toml` (1.97.0, kèm target `thumbv7m-none-eabi`); target mặc định của
  `cargo` ở `.cargo/config.toml`. Nâng Rust hay Embassy có chủ đích: build, `tools/kich_thuoc.py --kiem`, test, rồi cập
  nhật số trong README các bài. Cần thêm `arm-none-eabi-objcopy` (`nap.py` tìm theo `ARM_TOOLCHAIN`, rồi toolchain của
  PlatformIO, rồi `PATH`).
- **Trước khi commit:** `cargo build --release`, `cargo test -p ak-boot --target <target của máy>`,
  `cargo clippy --release -- -D warnings`, `python tools/kich_thuoc.py --kiem`. CI (`.github/workflows/ci.yml`) chạy
  đủ bốn bước. Job `rust-proto` của ak-mcu-base cũng lấy repo này: đổi `ak-boot` hay `ak-ffi` là đổi firmware C.
- Test tính chất ở `ak-boot/tests/tinh_chat.rs` (`proptest`, chỉ là dev-dependency, không vào firmware). Thấy file
  `*.proptest-regressions` sinh ra khi test hỏng thì giữ lại và commit nếu lỗi là thật (proptest chạy lại ví dụ đó trước).
- Nạp một bài: `python tools/nap.py <bài> --com <cổng>` (qua cổng COM, ~9 s) hoặc không `--com` để nạp ST-Link
  `mode=UR … -rst`; `--chi-build` để chỉ build. Test phần logic: `cargo test -p ak-boot --target <target của máy>`
  (ví dụ `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-gnu`).
- Nạp tay: `STM32_Programmer_CLI -c port=SWD mode=UR -d blink.img 0x08003000 -v -rst`. Đầu dò ST-Link tự thiết kế không có
  NRST/SWO; `mode=HOTPLUG` báo `Unable to get core ID` trên kit đang chạy, `mode=UR` thì được. Gặp
  `Cannot identify the device` khi đổi đầu dò → kiểm firmware đầu dò trước (cần ≥ V2J48M35).
- Nạp, xoá flash, reset board là thao tác vật lý: nói rõ và chờ đồng ý; board đang có firmware thì dump nguyên chip
  vào `fw_backup/` trước.

## Git

- Repo cá nhân, công khai: không đưa tên công ty, liên hệ, tên sản phẩm thương mại vào mã và tài liệu.
- Không commit output build (`target/`, `*.bin`, `*.img`).
