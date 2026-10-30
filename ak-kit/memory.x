/* AK MCU KIT (STM32L151CB, 32 KB RAM) chay sau bootloader ak-mcu-base 1.4.0 build 32K:
 * 0x08003000 la header anh 256 B (tools/mkimage.py them vao), bang vector o 0x08003100.
 * 256 B dau RAM de danh cho .noinit cua bootloader (lenh nhay, ly do reset). */
MEMORY
{
  FLASH : ORIGIN = 0x08003100, LENGTH = 116K - 256
  RAM   : ORIGIN = 0x20000100, LENGTH = 32K - 256
}
