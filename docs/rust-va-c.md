# Rust và C trên AK MCU KIT — so sánh từ một dự án thật

> Viết 30/10/2026. Mọi con số và mọi lỗi trong bài đều đo hoặc gặp trên **cùng một kit** (STM32L151CB, Cortex-M3
> 32 MHz, 128 KB flash, 32 KB RAM), **cùng một bootloader** (ak-mcu-base 1.4.0), trong hai repo
> [ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base) (C) và [ak-mcu-rust](https://github.com/hohoanganh/ak-mcu-rust)
> (Rust). Không có số liệu lấy từ nơi khác.

## Tóm tắt

| | C (ak-mcu-base) | Rust (ak-mcu-rust, Embassy) |
|---|---|---|
| Chạy trên chip nhỏ | ✓ | ✓ — bài lớn nhất 37 KB flash, 5,3 KB RAM |
| Nhiều việc cùng lúc | kernel AK: task + message + timer | `async`/`.await`, mỗi việc là một hàm đọc từ trên xuống |
| Đọc vượt mảng | đọc rác, chạy tiếp | dừng ngay, báo đúng tệp và dòng |
| Lỗi bộ nhớ, tranh chấp dữ liệu | phát hiện khi chạy (nếu may) | phần lớn bị chặn khi biên dịch |
| Lỗi phần cứng, thời gian, cấu hình chip | tự lo | **cũng tự lo** — Rust không giúp |
| Cùng một module (bộ tách khung `fw_proto`) | 58 384 B flash cả firmware | 58 696 B — **+312 B** |
| Test một module với hàng nghìn đầu vào ngẫu nhiên | tự viết vòng lặp, hoặc dựng libFuzzer + ASan | `proptest`: một khai báo, tự thu nhỏ ví dụ hỏng (mục 1.5) |
| Độ chín của công cụ cho STM32L1 | rất chín (SPL, CubeIDE) | dùng được, nhưng còn vấp (xem mục 5) |
| Người mới học | dễ bắt đầu, khó viết đúng | khó bắt đầu, khó viết sai |

**Kết luận ngắn:** giữ nền C đang chạy tốt; viết **module mới ở chỗ nhận dữ liệu từ bên ngoài** bằng Rust và trộn vào
firmware C — chi phí vài trăm byte, đã chạy trên kit và qua CI.

---

## 1. Cùng một việc, hai cách viết

### 1.1 Đọc vượt mảng

```c
/* C */
int diem[5] = {7, 9, 5, 10, 8};
int gia_tri = diem[5];      /* biên dịch được, chạy được: đọc 4 byte ngay sau mảng */
```

```rust
// Rust — bài 5, an-toan/src/main.rs
let diem = [7, 9, 5, 10, 8];
let gia_tri = diem[chi_so]; // chi_so = 5: chương trình dừng ở đúng dòng này
```

Trên kit, bài 5 hiện lên OLED:

```
PANIC
main.rs dong 34
index out of bounds: the len is 5 but the index is 5
```

C không sai về cú pháp — nó **tin người viết**. `diem[5]` trả về một con số trông hợp lệ, chương trình chạy tiếp; nếu
là **ghi** vượt mảng thì biến nằm cạnh bị hỏng âm thầm. Kiểu lỗi này thường chỉ lộ ra sau nhiều tuần chạy ngoài hiện
trường. Rust không làm lỗi biến mất — nó làm lỗi **lộ ra ngay, đúng chỗ**.

### 1.2 Nhiều việc cùng lúc

Bài 2: LED đổi mỗi 300 ms, còi kêu mỗi giây, OLED vẽ lại mỗi 100 ms.

```c
/* C với kernel AK (rút gọn): mỗi việc là một task xử lý message, nhịp do timer mềm gửi tới */
void task_led(ak_msg_t* msg) {
	switch (msg->sig) {
	case SIG_LED_TOGGLE:
		led_toggle();
		break;
	}
}
/* ... lúc khởi tạo: */
timer_set(TASK_LED_ID, SIG_LED_TOGGLE, 300, TIMER_PERIODIC);
```

```rust
// Rust — bài 2, ba-viec-cung-luc/src/main.rs
#[embassy_executor::task]
async fn nhay_led(mut led: Led) {
    loop {
        led.dao();
        Timer::after_millis(300).await; // chờ mà không chặn hai việc kia
    }
}
```

Cả hai đều **không cần RTOS** và đều chạy trên một lõi. Khác ở cách đọc: bản C chia một việc thành "ai gửi tín hiệu,
ai nhận, ở trạng thái nào"; bản Rust viết việc đó **tuần tự như một vòng lặp**, trình biên dịch tự cắt thành máy trạng
thái. Việc càng nhiều bước (chờ A rồi làm B rồi chờ C), khoảng cách càng rõ.

Kernel AK có thứ Rust/Embassy chưa có sẵn: ưu tiên giữa các task, giám sát task treo, nhật ký sự cố trong EEPROM.

### 1.3 Trao đổi giữa các việc

```c
/* C: message của kernel AK — nội dung là byte, người nhận tự ép kiểu */
task_post_common_msg(TASK_UI_ID, SIG_TIME, (uint8_t*)&gio, sizeof(gio));
```

```rust
// Rust — bài 4, dong-ho/src/main.rs
static KENH: Channel<CriticalSectionRawMutex, Gio, 4> = Channel::new();
KENH.send(gio).await;              // bên gửi
let gio = KENH.receive().await;    // bên nhận: chắc chắn là kiểu Gio
```

Cùng một ý tưởng — hai task không dùng chung biến, chỉ gửi thư cho nhau. Bản Rust **kiểm kiểu lúc biên dịch**: gửi
nhầm kiểu là không build được; bản C để chuyện đó cho người viết.

### 1.4 Báo lỗi

```c
/* C: mã trả về, dễ quên kiểm */
uint8_t kit_rtc_get(uint8_t* hh, uint8_t* mm, uint8_t* ss);   /* 1 = ok, 0 = lỗi */
kit_rtc_get(&h, &m, &s);   /* quên kiểm: h, m, s là rác, chương trình chạy tiếp */
```

```rust
// Rust — ak-kit/src/dong_ho.rs và bài 4
pub fn doc(&mut self) -> Result<Gio, LoiDongHo>;

match dong_ho.doc() {
    Ok(gio) => KENH.send(gio).await,
    Err(LoiDongHo::MatGio) => dong_ho.dat(Gio { gio: 0, phut: 0, giay: 0 }).ok(),
    Err(loi) => in_ra!("loi doc dong ho: {:?}", loi),
}
```

Muốn lấy được `Gio` thì **buộc phải** đi qua nhánh lỗi. Trên kit thật, chính nhánh `MatGio` đã chạy: RTC đã mất giờ
(cờ "dao động từng dừng"), bài 4 đặt lại 00:00:00 rồi chạy tiếp thay vì hiện giờ rác.

### 1.5 Test tính chất: một điều phải đúng với mọi đầu vào

Test thường kiểm vài ví dụ viết tay. Test tính chất (`proptest`) kiểm **một điều phải luôn đúng**: thư viện tự sinh
hàng nghìn đầu vào ngẫu nhiên. Khi điều đó sai, nó tự thu nhỏ đầu vào xuống ví dụ nhỏ nhất vẫn hỏng rồi in ra. Bộ tách
khung `fw_proto` và `boot_ctrl` của `ak-boot` (mã đang chạy thật trong firmware C, mục 2.2) có 7 tính chất như vậy, ở
[`ak-boot/tests/tinh_chat.rs`](../ak-boot/tests/tinh_chat.rs):

```rust
/// Mat dien giua luc ghi, o BAT KY byte nao: lan khoi dong sau doc ra hoac trang thai
/// cu, hoac trang thai moi — khong bao gio la mot thu lai giua hai cai.
#[test]
fn mat_dien_o_byte_bat_ky_van_con_trang_thai_cu(
    e in eeprom(), cu in trang_thai(), moi in trang_thai(), cat in 0usize..=20,
) { /* luu(cu); luu(moi) nhung mat dien sau `cat` byte; doc() phai ra cu (hoac moi neu cat == 20) */ }
```

| Tính chất | Đầu vào ngẫu nhiên |
|---|---|
| Đóng gói rồi tách lại ra đúng khung ban đầu | lệnh, số thứ tự, 0–132 B dữ liệu, chữ của shell đứng trước, đồng hồ ms sắp tràn |
| Byte rác không làm sập bộ tách; khung nào nó nhận ra cũng hợp lệ | tới 2 000 byte bất kỳ, khoảng cách thời gian bất kỳ |
| Sau rác bất kỳ, im lặng quá 500 ms thì khung kế tiếp chắc chắn tách được | rác + một khung |
| Lật một bit bất kỳ thì khung bị bỏ | vị trí và bit bất kỳ |
| Mã hoá rồi giải mã `boot_ctrl` ra đúng trạng thái ban đầu | mọi trường |
| EEPROM chứa gì cũng được (rác, ô hỏng, `seq` lệch xa): lưu xong đọc lại đúng | hai ô 32 B |
| Mất điện ở byte bất kỳ khi ghi: còn nguyên trạng thái cũ | EEPROM, trạng thái cũ/mới, chỗ mất điện |

Mỗi tính chất chạy 2 000 lần, cả bộ mất khoảng 2 giây trên máy tính. **Chúng có bắt được lỗi không?** Mình cố tình cài
hai lỗi rồi chạy lại:

- `luu()` ghi đè lên ô mới nhất thay vì ô kia (lỗi hai ô xen kẽ mất tác dụng): 2 tính chất của `boot_ctrl` hỏng.
  `proptest` thu nhỏ còn một ví dụ: trạng thái cũ chỉ khác mặc định ở `install_count: 1`, mất điện giữa lúc ghi, đọc lại
  ra trạng thái rỗng.
- CRC bỏ sót byte dữ liệu cuối: 2 tính chất của bộ tách khung hỏng, ví dụ thu nhỏ là `cmd = 0, seq = 0`, dữ liệu một byte.

Trên mã thật, `proptest` không tìm ra lỗi nào trong `ak-boot`. Nó lại tìm ra một lỗi trong **chính test**: lần chạy
thứ hai, tính chất "mất điện" hỏng với ví dụ thu nhỏ `cat = 19`, tức mất điện khi đã ghi 19/20 byte. Byte còn lại trong ô
là byte cũ, nhưng **tình cờ trùng** byte CRC cuối của bản ghi mới (xác suất 1/256). Thế là EEPROM đã chứa nguyên bản ghi
mới, và `boot_ctrl` đọc ra trạng thái mới là đúng. Test viết tay ban đầu cho rằng "chưa ghi đủ 20 byte thì phải ra trạng
thái cũ", và chưa ai nghĩ tới trường hợp này. Ví dụ đó được lưu trong `tinh_chat.proptest-regressions` và chạy lại trước
mỗi lần test.

Giá trị chính của bộ test là **cái lưới**: lần sau sửa bộ tách khung hay `boot_ctrl`, lỗi kiểu trên lộ ra trong 2 giây
chứ không phải trên kit.

So cho công bằng: bản C cũng test mất điện kỹ. `tests/test_fw.c` của ak-mcu-base cắt điện ở **mọi** thao tác flash
(dùng `setjmp`), và chạy dưới ASan. Khác biệt nằm ở hai chỗ:

- **Công sức.** Bên C, vòng cắt điện và bộ sinh đầu vào đều tự viết. Bên Rust, `proptest` là một dòng trong
  `Cargo.toml`, và việc thu nhỏ ví dụ hỏng có sẵn.
- **Điều được đảm bảo.** Tính chất "byte rác không làm sập" bên Rust đúng **cả trên chip**: mọi chỉ số mảng đều được
  kiểm, sai thì panic có tệp và dòng. Bên C, ASan chỉ bắt tràn bộ đệm trên máy tính; trên chip, tràn là đọc/ghi rác
  âm thầm.

---

## 2. Số đo trên kit

### 2.1 Kích thước các bài Rust (Embassy, sau bootloader)

| Bài | Nội dung | Flash | RAM tĩnh |
|---|---|---|---|
| 0 `blink` | ghi thẳng thanh ghi, không thư viện | 1 604 B | 4 B |
| 1 `hello-async` | LED + đếm giây, 2 task | 29 192 B | 3 256 B |
| 2 `ba-viec-cung-luc` | LED + còi + OLED, 3 task | 32 404 B | 5 440 B |
| 3 `nut-bam` | đàn ba phím, `select3` | 31 532 B | 3 272 B |
| 4 `dong-ho` | RTC + đồng hồ kim, channel | 37 032 B | 5 400 B |
| 5 `an-toan` | đọc vượt mảng, PANIC | 31 808 B | 3 296 B |
| 6 `cua-ferris` | trò chơi 30 khung/giây | 34 160 B | 3 424 B |

Đo bằng `python tools/kich_thuoc.py` sau `cargo build --release` (Rust 1.97.0, `opt-level = "s"`). CI chạy lại mỗi lần
push và báo lỗi nếu bảng vector vượt 256 B hay RAM đè lên vùng của bootloader.

Bài 1 chỉ nháy LED mà tốn 29 KB vì `ak_kit::init()` dựng sẵn **mọi** phần cứng của kit cho mọi bài: OLED, còi, nút,
RTC, màn hình PANIC (kéo theo font chữ) và phần nghe lệnh nạp qua cổng COM (driver EEPROM). Lúc `ak-kit` mới có LED và
cổng COM, bài 1 chỉ 9,9 KB. Đó là cái giá của một thư viện "lắp sẵn mọi thứ" — tiện cho người học, không hợp cho sản
phẩm, nơi chỉ nên kéo vào thứ thật sự dùng. Còn trên 116 KB của vùng app thì vẫn dư.

`opt-level = "z"` (ưu tiên kích thước hơn nữa) bớt 1,0–1,5 KB mỗi bài Embassy (3–5 %), RAM không đổi. Repo vẫn giữ `"s"`: các
bài không thiếu chỗ, và I2C giả lập của OLED nhạy với tốc độ mã (mục 4), nên chỉ đổi khi đã thử trên kit.

Phần nặng nhất là định dạng số thực: bản đầu của bài 4 in nhiệt độ bằng `{:.1}` tốn thêm **15 KB** so với in số nguyên.

### 2.2 Cùng một module: bộ tách khung `fw_proto`

Module nhận khung lệnh nạp firmware (`A5 | cmd | seq | len | dữ liệu | crc16`) — chỗ nhận byte từ bên ngoài, nơi dễ tràn
bộ đệm nhất. Bản Rust thay bản C bên trong firmware demo của ak-mcu-base (tùy chọn `AK_RUST_PROTO`):

| | Bản C | Bản C + Rust |
|---|---|---|
| Flash cả firmware demo | 58 384 B | 58 696 B (**+312 B**) |
| RAM | 7 724 B | 7 996 B (+272 B) |
| Dòng mã hàm nhận byte | 81 | 54 |
| Test | qua test chung của `fw_proto` | 14 test ví dụ + 7 test tính chất (`cargo test`), gồm so hành vi với bản C |
| Tải 58 KB qua cổng COM trên kit | 8,5 s (bộ tách khung C của bootloader) | 8,7 s (bộ tách khung Rust của app) |
| `make test` của ak-mcu-base | 13/13 | 13/13 (CI job `rust-proto`) |

Lần link đầu, bản Rust tốn thêm **2,9 KB** — không phải do mã Rust (phần thật chỉ ~0,5 KB) mà vì thư viện Rust mang
theo `memcpy`/`memmove` riêng, đè lên bản của newlib. Đặt `libc` trước `libak_ffi.a` khi link và trỏ `__aeabi_memcpy*`
về newlib thì còn +312 B.

### 2.3 Nạp lại firmware qua cổng COM (không cần ST-Link)

| Chuyển | Thời gian |
|---|---|
| bài Rust → bài Rust (32 KB) | 9 s cả quy trình |
| bài Rust → app C 1.3.11 (58 KB) | 8,5 s tải |
| app C → bài Rust (34 KB) | 5,0 s tải |

Cùng bootloader, cùng giao thức, cùng công cụ `ak_fw.py` — app C và bài Rust thay nhau được, chỉ qua cổng COM.

---

## 3. Rust đã bắt được gì cho chính dự án này

- **Mượn một biến hai lần cùng lúc** (`ak-kit/src/coi.rs`, lỗi E0499): giữ tham chiếu tới kênh PWM rồi gọi một hàm khác
  cũng cần kênh đó. Trong C đây là hai con trỏ cùng trỏ vào một thanh ghi — chạy được, sai lúc nào không biết. Rust từ
  chối build cho tới khi tách phạm vi.
- **Đọc vượt mảng lúc chạy** (bài 5): dừng đúng dòng thay vì đọc rác.
- **Kiểu dữ liệu giữa các task** (bài 4): channel chỉ nhận đúng kiểu `Gio`.
- **Lỗi bắt buộc phải xử lý** (`Result` khi đọc RTC, EEPROM, I2C): không có chỗ nào "quên kiểm mã trả về".
- **Một giả định sai trong test mất điện** (`ak-boot/tests/tinh_chat.rs`, mục 1.5): `proptest` tìm ra trường hợp byte
  cũ trùng byte mới mà người viết test không nghĩ tới. Lỗi nằm ở test, không ở firmware, nhưng cũng là loại chuyện chỉ
  máy sinh đầu vào mới tìm được.

## 4. Rust KHÔNG cứu được gì — các lỗi thật đã gặp

Mọi lỗi tốn thời gian nhất của dự án đều nằm **dưới** tầng ngôn ngữ. Viết bằng C cũng gặp y hệt:

| Lỗi | Hiện tượng | Nguyên nhân |
|---|---|---|
| Bảng vector 1 KB | không link được | bootloader đặt bảng vector ở `0x08003100` (chỉ căn 256 B); thư viện Rust mặc định khai 240 ngắt |
| Chip ngủ thì mất ST-Link | `Unable to get core ID`, đầu dò treo | lõi ngủ WFE quá lâu, đầu dò không có chân NRST để kéo ra |
| OLED không trả lời ở 32 MHz | không ACK ở bất kỳ địa chỉ nào | I2C giả lập kéo SCL lên ngay sau khi đổi SDA; dây SDA kéo lên yếu, chưa kịp lên mức cao |
| Mất byte lệnh nạp | khung lệnh hỏng khi OLED đang vẽ | USART của L151 chỉ giữ 1 byte; phải nhận bằng ngắt vào hàng đợi |
| Firmware phình 2,9 KB | khi trộn Rust vào C | thứ tự link, Rust mang `memcpy` riêng |

Rust làm phần mềm an toàn hơn; nó không biết điện trở kéo lên trên board yếu hay mạnh. Hiểu phần cứng vẫn là việc
của người viết firmware, bất kể ngôn ngữ.

## 5. Cái giá của Rust

- **Học khó hơn.** Quy tắc mượn (`&`, `&mut`), vòng đời, `async` — người quen C sẽ bị trình biên dịch từ chối nhiều lần
  trước khi build được lần đầu.
- **Hệ sinh thái cho dòng chip cũ còn mỏng.** Embassy chạy tốt trên STM32L1 nhưng ví dụ trên GitHub viết cho bản mới hơn
  bản phát hành (tên enum khác: `PllMul::Mul6` so với `PllMul::MUL6`); PAC `stm32l1` khai bảng ngắt theo dòng dung lượng lớn, dài hơn bảng của L151CB.
- **Thêm một bộ công cụ.** `rustup`, `cargo`, target `thumbv7m-none-eabi` — CI cũng phải cài thêm.
- **Hai ngôn ngữ trong một sản phẩm** là gấp đôi thứ người bảo trì phải biết.
- **Định dạng số thực đắt** (mục 2.1) — với C dùng `xprintf` tự viết thì rẻ hơn nhiều.

## 6. Khi nào chọn gì

| Tình huống | Chọn |
|---|---|
| Firmware đang chạy ổn, đã có test, có OTA an toàn | **giữ C** — viết lại không đem thêm gì |
| Module mới **nhận dữ liệu từ bên ngoài** (giao thức, gói tin, tệp cấu hình) | **Rust, trộn vào C** — rẻ (+312 B), test được trên máy tính |
| Dự án mới, chip mới (nRF52, STM32 dòng mới), người viết sẵn sàng học | **Rust + Embassy** từ đầu |
| Dạy người mới về nhúng | Rust cho bài học về an toàn và `async`; C để hiểu chip chạy thế nào (bài 0 — ghi thẳng thanh ghi — là cầu nối) |
| Sản phẩm chạy pin, cần ngủ sâu | được cả hai; với Rust dùng bộ chạy `platform-cortex-m` và tính trước cách nạp lại (mục 4) |

## Đọc tiếp

- Chuỗi 7 bài cho người mới: [README](../README.md) — mỗi bài một ý, có hướng dẫn quay clip.
- Lộ trình và các cổng quyết định: [lo-trinh.md](lo-trinh.md).
- Bẫy kỹ thuật chi tiết: [CLAUDE.md](../CLAUDE.md).
- Phần Rust trong firmware C: tùy chọn `AK_RUST_PROTO` của [ak-mcu-base](https://github.com/hohoanganh/ak-mcu-base),
  crate [`ak-ffi`](../ak-ffi/) và [`ak-boot`](../ak-boot/).
