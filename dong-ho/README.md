# Bài 4 — Các task nói chuyện qua channel

**Câu chốt:** hai task không dùng chung biến nào — task đọc đồng hồ `send` vào `KENH`, task hiển thị `receive` từ
`KENH`. Giống message của kernel AK trong firmware C, nhưng trình biên dịch kiểm luôn kiểu dữ liệu gửi qua.

- `doc_dong_ho`: đọc RTC PCF85063 (I2C1, 0x51) 5 lần mỗi giây, chỉ gửi khi sang giây mới.
- `hien_thi`: nhận giờ, vẽ đồng hồ kim (giờ, phút, giây) bên trái và giờ số bên phải lên OLED, in `HH:MM:SS` ra cổng COM.
- RTC báo đã mất giờ (cờ "dao động từng dừng") thì đặt lại 00:00:00 và chạy tiếp — lỗi đi qua `Result`, không bị bỏ
  qua âm thầm.

## Nạp

```bash
python tools/nap.py dong-ho
```

## Quay clip (30–45 s)

1. Cận cảnh OLED: kim giây quay từng nấc, giờ số đổi theo.
2. Mã nguồn, khoanh:
   - dòng 25: `static KENH: Channel<…, Gio, 4>` — hàng đợi chứa tối đa 4 lần giờ;
   - dòng 41: `KENH.send(gio).await` — task đọc gửi đi;
   - dòng 57: `KENH.receive().await` — task hiển thị chờ tới khi có giờ mới;
   - dòng 30–31: hai task được giao hai phần cứng riêng (`kit.dong_ho`, `kit.man_hinh`) — không ai tranh của ai.
3. Câu kết: "Thử gửi nhầm kiểu dữ liệu vào KENH xem — chương trình không build được."

## Kích thước

| | |
|---|---|
| Flash | 31 608 B (đồ họa + sin/cos của `micromath`) |
| RAM tĩnh | 4,5 KB |

## Trạng thái

Chạy trên kit (30/10/2026): đếm đều mỗi giây trên cổng COM. RTC trên kit đã mất giờ nên bắt đầu từ 00:00:00.

Ghi chú: bản đầu của bài này đọc cảm biến SHT45 (đồ thị nhiệt độ); kit không gắn SHT45 ở header J8 nên đổi sang RTC.
