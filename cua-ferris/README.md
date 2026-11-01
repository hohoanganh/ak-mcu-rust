# Bài 6 — Ferris hứng sao

**Câu chốt:** Rust vẽ mượt trên chip 32 MHz — 30 khung mỗi giây, mỗi khung vẽ mất khoảng 3 ms.

Trò chơi nhỏ, bài kết của chuỗi. Gom lại mọi thứ các bài trước đã dạy:

| Thứ đã học | Ở bài 6 |
|---|---|
| Task + `.await` (bài 1, 2) | vòng khung hình `Timer::after_millis(33).await` |
| Nút (bài 3) | ▼ đi trái, ⏎ đi phải, ▲ nhảy — đọc mỗi khung bằng `dang_bam()` |
| Channel (bài 4) | âm thanh chạy ở task riêng, nhận lệnh qua `AM_THANH` — còi kêu mà hình không khựng |
| An toàn (bài 5) | không có chỉ số nào tự tính tay vào mảng: hình Ferris duyệt bằng iterator |

Hình **Ferris** (linh vật của Rust) vẽ bằng điểm ảnh ngay trong mã: mỗi dòng của `FERRIS` là một chuỗi `".##..."`.
Sao rơi từ trên xuống: hứng được thì +1 và còi kêu cao; để rơi xuống đất thì còi kêu trầm.

## Nạp

```bash
python tools/nap.py cua-ferris
```

## Quay clip (30–60 s)

Còi mặc định tắt. Muốn có tiếng thì thêm `kit.coi.dat_am_luong(50);` ngay sau `ak_kit::init(spawner)` (1 = nhỏ
nhất, 50 = to nhất).

1. Mở đầu bằng mã nguồn, dòng 23–38: mảng `FERRIS` — nhìn chuỗi ký tự đã thấy con cua.
2. Cận cảnh OLED, bật tiếng: chơi vài lượt, nhảy hứng sao, để rơi một ngôi sao cho nghe tiếng trầm.
3. Khoanh:
   - dòng 48: `spawner.spawn(phat_am_thanh(...))` — task âm thanh riêng;
   - dòng 62, 74, 76: `AM_THANH.try_send(...)` — vòng chơi chỉ gửi lệnh, không chờ còi;
   - dòng 65: `van_toc_y += 0.35` — một dòng là có trọng lực;
   - dòng 101–107: duyệt `FERRIS`, mỗi `'#'` thành một điểm sáng.
4. Câu kết của cả chuỗi: "Sáu bài, một kit 32 MHz, không RTOS — và chưa có một lần đọc nhầm bộ nhớ nào lọt qua."

## Kích thước

Đo 30/10/2026, sau khi `ak-kit` có màn hình PANIC và phần nạp qua cổng COM.

| | |
|---|---|
| Flash | 34 160 B |
| RAM tĩnh | 3,3 KB |
| Vẽ một khung | ~3 ms (khung 33 ms) |

## Trạng thái

Chạy trên kit (30/10/2026): ~3 ms/khung đo qua cổng COM.
