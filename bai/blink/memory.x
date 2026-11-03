/* AK MCU KIT, STM32L151CB, chạy sau bootloader ak-mcu-base:
 * 0x08003000 là header ảnh 256 B (tools/mkimage.py build thêm vào),
 * bảng vector của app bắt đầu ở 0x08003100.
 * 256 B đầu RAM để dành cho .noinit của bootloader (lệnh nhảy, lý do reset). */
MEMORY
{
  FLASH : ORIGIN = 0x08003100, LENGTH = 116K - 256
  RAM   : ORIGIN = 0x20000100, LENGTH = 16K - 256
}
