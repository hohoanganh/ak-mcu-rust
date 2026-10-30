# Bài 3 — Chờ sự kiện, không hỏi liên tục

**Câu chốt:** `select3(nút ▼, nút ▲, nút ⏎).await` — chờ cả ba nút trong một dòng; nút nào bấm trước thì nhánh đó chạy.
Không có vòng lặp nào đọc chân nút: ngắt EXTI đánh thức đúng chỗ đang chờ.

"Đàn ba phím": ▼ = Đô, ▲ = Mi, ⏎ = Sol. Mỗi lần bấm: còi phát nốt, LED sáng trong lúc kêu, OLED hiện tên nốt và số lần
bấm, cổng COM in `bam lan N: <nốt>`.

## Nạp

```bash
python tools/nap.py nut-bam
```

## Quay clip (30–45 s)

1. Cận cảnh kit, bật tiếng: bấm một giai điệu ngắn (ví dụ Đô–Mi–Sol–Mi–Đô); màn hình đổi theo từng nốt.
2. Mã nguồn, khoanh:
   - dòng 28–33: `select3(...).await` — một dòng chờ ba nút;
   - dòng 35–37: ba nhánh `Either3`, mỗi nút một nốt;
   - `ak-kit/src/nut.rs`, hàm `cho_bam()`: lọc dội phím 20 ms bằng `Timer`, cũng là `.await`.
3. Câu kết: "Không có `while (1) { if (nut1) … if (nut2) … }`: chương trình chỉ chạy khi có người bấm."

**Ghi chú trung thực:** các bài dùng bộ chạy task `platform-spin` (không cho chip ngủ, để ST-Link của kit luôn nạp lại
được), nên CPU vẫn quay vòng chứ không thật sự nghỉ. Sản phẩm chạy pin dùng `platform-cortex-m` — bộ chạy đó ngủ WFE
cho tới khi ngắt nút đánh thức.

## Kích thước

| | |
|---|---|
| Flash | 25 152 B |
| RAM tĩnh | 2,4 KB |

## Trạng thái

Build và nạp được (30/10/2026). **Chưa kiểm phần bấm nút trên kit.**
