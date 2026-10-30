# Bài 1 — Chờ mà không chặn

**Câu chốt:** `Timer::after(...).await` — chờ mà không bắt CPU ngồi đợi, việc khác vẫn chạy.

Hai việc chạy cùng lúc trên một lõi, không RTOS: LED nháy 4 lần mỗi giây (task `nhay_led`), còn `main` đếm giây và in
ra cổng COM.

## Nạp

```bash
python tools/nap.py hello-async
```

Mở cổng COM ảo của ST-Link ở 115200 baud:

```
Xin chao tu Embassy! (Rust async tren AK MCU KIT)
da chay 1 giay
da chay 2 giay
...
```

## Quay clip (30–40 s)

1. Màn hình máy tính: chạy lệnh nạp, rồi cửa sổ cổng COM hiện dòng chào và số giây tăng dần.
2. Cận cảnh kit: LED nháy nhanh trong khi số giây trên màn hình vẫn đều đặn — hai việc, hai nhịp khác nhau.
3. Kết bằng mã nguồn, khoanh:
   - dòng 22 `spawner.spawn(nhay_led(...))` — giao việc cho task riêng;
   - dòng 27 và 37 `Timer::after_...().await` — "chờ" ở hai chỗ, không chỗ nào chặn chỗ nào;
   - dòng 33 `#[embassy_executor::task]` — một hàm `async` trở thành một task.

## Kích thước

| | |
|---|---|
| Flash | 9 884 B (ảnh nạp có header: 10 196 B) |
| RAM tĩnh | 228 B (`.data` 56 + `.bss` 172) |

## Đã kiểm trên kit (30/10/2026)

Chạy sau bootloader ak-mcu-base 1.4.0; nhịp đếm 0,999 s/dòng (đo phía máy tính); LED nháy đều (Hoàng Anh nhìn);
ST-Link nạp lại được trong lúc chương trình chạy.
