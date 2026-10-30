# CLAUDE.md

Ghi chú cho Claude khi làm việc trong repo này. Trả lời và viết tài liệu bằng tiếng Việt.

## Repo này là gì

Thử nghiệm firmware Rust trên AK MCU KIT (STM32L151), **chạy sau bootloader của
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
  **45 ngắt** → `embassy-stm32` với feature chip đúng sẽ ra bảng 244 B (chưa kiểm bằng build).
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
  RAM**, không phải 16 KB như ak-mcu-base đang khai. Chưa kiểm trên kit: đừng nới `memory.x` lên 32 KB trước khi đọc
  được chữ trên chip hoặc thử ghi/đọc `0x20004000`.

## Build, nạp

- Máy `VTNN Tu Cang`: Rust ở `D:\devcache\cargo` (toolchain `stable-x86_64-pc-windows-gnu`, target
  `thumbv7m-none-eabi`); `arm-none-eabi-objcopy` lấy từ `D:\devcache\platformio\packages\toolchain-gccarmnoneeabi\bin`.
  Repo nằm ở `D:\dev\ak-mcu-rust` — ngoài OneDrive.
- Nạp: `STM32_Programmer_CLI -c port=SWD mode=UR -d blink.img 0x08003000 -v -rst`. Đầu dò ST-Link tự thiết kế không có
  NRST/SWO; `mode=HOTPLUG` báo `Unable to get core ID` trên kit đang chạy, `mode=UR` thì được. Gặp
  `Cannot identify the device` khi đổi đầu dò → kiểm firmware đầu dò trước (cần ≥ V2J48M35).
- Nạp, xoá flash, reset board là thao tác vật lý: nói rõ và chờ đồng ý; board đang có firmware thì dump nguyên chip
  vào `fw_backup/` trước.

## Git

- Repo cá nhân, công khai: không đưa tên công ty, liên hệ, tên sản phẩm thương mại vào mã và tài liệu.
- Không commit output build (`target/`, `*.bin`, `*.img`).
