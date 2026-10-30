# Bài 5 — Lỗi C âm thầm, Rust dừng lại và chỉ đúng dòng

**Câu chốt:** đọc vượt mảng — C trả về giá trị rác rồi chạy tiếp, Rust dừng ngay và chỉ ra tệp, số dòng, lý do.

Mảng `diem` có 5 phần tử (chỉ số 0..4). ▲ tăng chỉ số và **cố tình không chặn ở 4**, ▼ giảm, ⏎ đọc `diem[chi_so]`.
Ở chỉ số 5, bấm ⏎:

```
┌──────────────────────┐
│        PANIC         │  ← dải trắng
│ main.rs dong 34      │
│ index out of bounds: │
│ the len is 5 but the │
│ index is 5           │
└──────────────────────┘
```

Cổng COM cũng in `panic: … tai an-toan\src\main.rs:34`, LED nháy nhanh. Bấm RESET để chạy lại. Phần hiện panic nằm trong
`ak-kit`, nên mọi bài panic đều hiện lên OLED như vậy.

## So với C

```c
int diem[5] = {7, 9, 5, 10, 8};
int gia_tri = diem[5];   // biên dịch được, chạy được: đọc 4 byte ngay sau mảng
```

C không kiểm giới hạn mảng: `diem[5]` là ô nhớ của biến khác (hoặc rác), chương trình chạy tiếp với giá trị sai —
kiểu lỗi chỉ lộ ra sau nhiều tuần ngoài hiện trường. Nếu là **ghi** vượt mảng, C còn âm thầm làm hỏng biến bên cạnh.

## Nạp

```bash
python tools/nap.py an-toan
```

## Quay clip (30–45 s)

1. Cận cảnh OLED: ⏎ ở `diem[0]`, `diem[4]` — đọc được, còi píp.
2. ▲ lên `diem[5]`, màn báo `NGOAI MANG!`. Dừng một nhịp — "C sẽ làm gì ở đây?"
3. ⏎ → màn hình PANIC chỉ đúng `main.rs dong 34`.
4. Mã nguồn, khoanh dòng 23 (`let diem = [...]`, 5 phần tử), dòng 31 (`chi_so += 1` không chặn) và dòng 34
   (`diem[chi_so]` — đúng dòng màn hình báo).
5. Câu kết: "Rust không làm lỗi biến mất — nó làm lỗi lộ ra ngay, đúng chỗ."

## Kích thước

Đo 30/10/2026, sau khi `ak-kit` có màn hình PANIC và phần nạp qua cổng COM.

| | |
|---|---|
| Flash | 31 716 B |
| RAM tĩnh | 3,2 KB |

## Trạng thái

Chạy trên kit (30/10/2026): bấm ⏎ ở `diem[5]`, OLED hiện PANIC chỉ đúng dòng 34 (Hoàng Anh xác nhận).
