# Bài 2 — Ba việc cùng lúc, không cần RTOS

**Câu chốt:** ba task, ba nhịp khác nhau, chung một lõi CPU — mỗi chỗ `.await` là một chỗ nhường lượt.

| Task | Việc | Nhịp |
|---|---|---|
| `nhay_led` | đổi trạng thái LED | 300 ms |
| `tich_tac` | còi kêu "tích" 15 ms | 1 s |
| `dem_gio` | vẽ đồng hồ đếm lên OLED | 100 ms |

## Nạp

```bash
python tools/nap.py ba-viec-cung-luc
```

## Quay clip (30–45 s)

1. Cận cảnh kit, bật tiếng: LED nháy, còi tích tắc mỗi giây, số trên OLED chạy 0.1 s một lần — ba nhịp lệch nhau
   rõ ràng.
2. Chuyển sang mã nguồn, khoanh:
   - dòng 25–27: ba lần `spawner.spawn(...)` — giao ba việc cho ba task;
   - dòng 34, 42, 64: ba chỗ `Timer::after_...().await` với ba thời gian khác nhau;
   - dòng 41: `coi.keu(2000, 15).await` — đang kêu cũng không chặn LED hay màn hình.
3. Câu kết: "Trong C phải tự viết vòng lặp so thời gian cho từng việc, hoặc dùng RTOS. Ở đây mỗi việc là một hàm
   đọc từ trên xuống."

## Kích thước

Đo 30/10/2026, sau khi `ak-kit` có màn hình PANIC và phần nạp qua cổng COM.

| | |
|---|---|
| Flash | 32 328 B (`embedded-graphics` + hai font chiếm phần lớn) |
| RAM tĩnh | 5,3 KB (hai bộ đệm khung OLED 1 KB mỗi cái) |
| Vẽ một khung | ~1,9 ms (chỉ gửi đoạn cột đã đổi); khung đầu tiên 25 ms |
