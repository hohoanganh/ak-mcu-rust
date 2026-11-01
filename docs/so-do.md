# Sơ đồ luồng chạy

Các sơ đồ dưới đây vẽ bằng [Mermaid](https://mermaid.js.org/): GitHub tự vẽ khi xem file. Muốn sửa thì sửa chữ trong
khối `mermaid`. Mỗi sơ đồ ghi tệp mã tương ứng để đối chiếu.

1. [Bộ nhớ: bootloader C và app Rust nằm ở đâu](#1-bộ-nhớ-bootloader-c-và-app-rust-nằm-ở-đâu)
2. [Từ lúc bật nguồn tới khi bài chạy](#2-từ-lúc-bật-nguồn-tới-khi-bài-chạy)
3. [Nhiều việc cùng lúc: kernel AK (C) và Embassy (Rust)](#3-nhiều-việc-cùng-lúc-kernel-ak-c-và-embassy-rust)
4. [Nạp bài mới qua cổng COM, chạy thử, quay lui](#4-nạp-bài-mới-qua-cổng-com-chạy-thử-quay-lui)
5. [Bộ tách khung `fw_proto`](#5-bộ-tách-khung-fw_proto)
6. [`boot_ctrl`: hai ô ghi xen kẽ, mất điện không mất trạng thái](#6-boot_ctrl-hai-ô-ghi-xen-kẽ-mất-điện-không-mất-trạng-thái)
7. [Rust nằm trong firmware C (`ak-ffi`)](#7-rust-nằm-trong-firmware-c-ak-ffi)
8. [Đọc vượt mảng: C chạy tiếp, Rust dừng đúng dòng](#8-đọc-vượt-mảng-c-chạy-tiếp-rust-dừng-đúng-dòng)
9. [Slave Modbus RTU với bộ tách yêu cầu Rust (`ak-modbus`)](#9-slave-modbus-rtu-với-bộ-tách-yêu-cầu-rust-ak-modbus)

## 1. Bộ nhớ: bootloader C và app Rust nằm ở đâu

Bài Rust không đè bootloader. Nó nằm trong vùng APP, nên nạp lại `app.img` của bản C lúc nào cũng được.

```mermaid
flowchart LR
    subgraph FLASH["Flash trong chip 128 KB"]
        direction TB
        B["0x08000000<br/>Bootloader C, 12 KB<br/>(ak-mcu-base)"]
        H["0x08003000<br/>Header ảnh 256 B<br/>CRC, tên board, phiên bản"]
        V["0x08003100<br/>Bảng vector 244 B<br/>(phải ≤ 256 B)"]
        A["Mã bài Rust<br/>tới 0x08020000"]
        B --- H --- V --- A
    end
    subgraph RAM["RAM 32 KB"]
        direction TB
        N["0x20000000<br/>.noinit của bootloader<br/>lệnh nhảy, lý do reset, sự cố"]
        R["0x20000100<br/>.data, .bss, stack của bài Rust"]
        N --- R
    end
    subgraph EE["EEPROM 0x08080000"]
        C["[0, 64): boot_ctrl<br/>hai ô 32 B"]
    end
    FLASH ~~~ RAM ~~~ EE
```

Mã: [`ak-kit/memory.x`](../ak-kit/memory.x), [`tools/kich_thuoc.py`](../tools/kich_thuoc.py) (CI kiểm bảng vector và
RAM mỗi lần push), [`ak-boot/src/boot_ctrl.rs`](../ak-boot/src/boot_ctrl.rs).

## 2. Từ lúc bật nguồn tới khi bài chạy

```mermaid
flowchart TD
    RST(["Bật nguồn / reset"]) --> BL["Bootloader C đọc boot_ctrl trong EEPROM"]
    BL --> CHK{"Ảnh ở vùng APP:<br/>CRC đúng, đúng tên board?"}
    CHK -- "không" --> LOADER["Ở lại bootloader, chờ ảnh qua cổng COM"]
    CHK -- "có" --> JMP["Nhảy vào app qua một lần reset<br/>VTOR = 0x08003100, MSI 2 MHz, watchdog tắt"]
    JMP --> INIT["cortex-m-rt: chép .data, xoá .bss, gọi main()"]
    INIT --> KIT["ak_kit::init(spawner)"]
    KIT --> NUT{"Đang giữ nút giữa PC13?"}
    NUT -- "có" --> NAP["Chế độ nạp: LED nháy chậm,<br/>không chạy Embassy → ST-Link luôn vào được"]
    NUT -- "không" --> HW["Xung nhịp 32 MHz, LED, cổng COM,<br/>còi, OLED, ba nút, RTC"]
    HW --> TASK["Chạy task nghe_lenh_nap<br/>(nhận lệnh nạp, tự xác nhận sau 30 s)"]
    TASK --> MAIN["Bài spawn các task của mình"]
    MAIN --> EXE["Executor Embassy (platform-spin):<br/>hỏi lần lượt task nào sẵn sàng, không ngủ"]
```

Mã: [`ak-kit/src/lib.rs`](../ak-kit/src/lib.rs) (`init`, `nut_giua_dang_giu`, `che_do_nap`), bài 1
[`hello-async/src/main.rs`](../hello-async/src/main.rs).

## 3. Nhiều việc cùng lúc: kernel AK (C) và Embassy (Rust)

Cả hai đều không có RTOS và chạy trên một lõi. Bên C, một việc nhiều bước bị chia thành signal và các lần gọi handler.
Bên Rust, cùng việc đó viết thành một vòng lặp đọc từ trên xuống; trình biên dịch tự cắt nó ở mỗi `.await`.

```mermaid
flowchart LR
    subgraph C["C: kernel AK"]
        direction TB
        T1["timer_set(TASK_LED_ID,<br/>SIG_LED_TOGGLE, 300, PERIODIC)"] --> Q[("Hàng đợi message")]
        Q --> D{"Kernel lấy message<br/>ưu tiên cao nhất"}
        D --> H["task_led(msg):<br/>switch (msg->sig)"]
        H -- "return" --> D
    end
    subgraph R["Rust: Embassy"]
        direction TB
        L["loop {<br/>led.dao();<br/>Timer::after_millis(300).await;<br/>}"] -- ".await: trả quyền" --> E{"Executor"}
        TIM["TIM2 báo hết 300 ms"] -- "đánh thức" --> E
        E -- "chạy tiếp ngay sau .await" --> L
    end
    C ~~~ R
```

Trong một bài có ba việc (bài 2), các task thay nhau chạy như sau:

```mermaid
sequenceDiagram
    participant E as Executor
    participant L as nhay_led
    participant K as coi
    participant M as man_hinh
    participant T as TIM2 (đồng hồ)
    E->>L: chạy
    L->>T: Timer::after_millis(300).await
    E->>K: chạy
    K->>T: Timer::after_millis(985).await
    E->>M: chạy, vẽ OLED
    M->>T: Timer::after_millis(100).await
    T-->>E: 100 ms: man_hinh sẵn sàng
    E->>M: chạy tiếp ngay sau .await
    T-->>E: 300 ms: nhay_led sẵn sàng
    E->>L: chạy tiếp, đổi LED
```

Mã: bài 2 [`ba-viec-cung-luc/src/main.rs`](../ba-viec-cung-luc/src/main.rs). So sánh chi tiết:
[rust-va-c.md mục 1.2](rust-va-c.md#12-nhiều-việc-cùng-lúc).

## 4. Nạp bài mới qua cổng COM, chạy thử, quay lui

`python tools/nap.py <bài> --com <cổng>` không cần ST-Link. Bài đang chạy (Rust hay C đều được) chỉ cần hiểu lệnh
LOADER; phần nhận và cài ảnh do bootloader C làm.

```mermaid
sequenceDiagram
    autonumber
    participant PC as Máy tính (nap.py, ak_fw.py)
    participant APP as Bài đang chạy (task nghe_lenh_nap)
    participant EE as EEPROM (boot_ctrl)
    participant BL as Bootloader C
    PC->>PC: cargo build, objcopy, mkimage.py (thêm header 256 B)
    PC->>APP: khung LOADER
    APP->>EE: boot_ctrl::dat_lenh(CMD_LOADER)
    APP-->>PC: trả lời OK
    APP->>BL: sys_reset()
    BL->>EE: đọc lệnh LOADER
    PC->>BL: ak_fw.py flash: gửi ảnh từng khối 128 B
    BL->>BL: kiểm CRC, chép ảnh cũ sang BACKUP, cài ảnh mới
    BL->>EE: trial = 1 (ảnh mới chạy thử)
    BL->>APP: nhảy vào bài mới
    alt chạy ổn 30 s
        APP->>EE: boot_ctrl::xac_nhan(): trial = 0
    else hỏng trước 30 s (panic, treo, reset)
        BL->>EE: mỗi lần khởi động: trial_boots + 1
        BL->>BL: trial_boots đủ 3: cài lại ảnh cũ từ BACKUP
    end
```

Mã: [`tools/nap.py`](../tools/nap.py), [`ak-kit/src/nap.rs`](../ak-kit/src/nap.rs),
[`ak-boot/src/boot_ctrl.rs`](../ak-boot/src/boot_ctrl.rs). Bootloader: `src/boot/boot_main.c` của ak-mcu-base. Đường
quay lui (nhánh `else`) chưa thử trên kit.

## 5. Bộ tách khung `fw_proto`

Khung: `A5 | cmd | seq | len (2 B) | dữ liệu | crc16 (2 B)`. Bộ tách nhận từng byte một. Byte nào không thuộc khung
(chữ gõ vào shell) thì để cho shell.

```mermaid
stateDiagram-v2
    [*] --> Sof
    Sof --> Cmd: byte A5
    Cmd --> Seq
    Seq --> DaiThap
    DaiThap --> DaiCao
    DaiCao --> Sof: len > 132 (bỏ khung)
    DaiCao --> CrcThap: len = 0
    DaiCao --> DuLieu: len 1..132
    DuLieu: DuLieu (nhận từng byte dữ liệu)
    DuLieu --> CrcThap: đủ len byte
    CrcThap --> CrcCao
    CrcCao --> Sof: CRC đúng → trả Some(khung)
    CrcCao --> Sof: CRC sai → bỏ
    note right of Sof
        Ở trạng thái bất kỳ, hai byte cách nhau quá 500 ms
        thì khung đang dở bị bỏ, quay về Sof.
        Ở Sof, byte khác A5 là của shell.
    end note
```

Mã: [`ak-boot/src/khung.rs`](../ak-boot/src/khung.rs). Test tính chất (byte rác, lật bit, im lặng 500 ms):
[`ak-boot/tests/tinh_chat.rs`](../ak-boot/tests/tinh_chat.rs).

## 6. `boot_ctrl`: hai ô ghi xen kẽ, mất điện không mất trạng thái

Mỗi lần lưu chỉ ghi vào ô **cũ hơn**, kèm `seq` tăng 1 và CRC-32. Mất điện giữa lúc ghi thì chỉ hỏng ô đang ghi; ô kia
vẫn giữ trạng thái trước đó.

```mermaid
flowchart TD
    S(["luu(trạng thái mới)"]) --> R["Đọc hai ô, bỏ ô sai magic hoặc sai CRC"]
    R --> W{"Ô nào mới nhất?<br/>(seq lớn hơn, tính vòng qua 255)"}
    W -- "ô 0" --> G1["Ghi vào ô 1, seq = seq cũ + 1"]
    W -- "ô 1" --> G0["Ghi vào ô 0, seq = seq cũ + 1"]
    W -- "không ô nào hợp lệ" --> G0
    G1 --> K["Đọc lại, so khớp"]
    G0 --> K
    G1 -. "mất điện giữa chừng" .-> X["Ô đang ghi sai CRC → bị bỏ<br/>doc() trả trạng thái cũ ở ô kia"]
    G0 -. "mất điện giữa chừng" .-> X
```

Mã: [`ak-boot/src/boot_ctrl.rs`](../ak-boot/src/boot_ctrl.rs). Tính chất "mất điện ở byte bất kỳ" được kiểm với hàng
nghìn trạng thái ngẫu nhiên: [rust-va-c.md mục 1.5](rust-va-c.md#15-test-tính-chất-một-điều-phải-đúng-với-mọi-đầu-vào).

## 7. Rust nằm trong firmware C (`ak-ffi`)

Không viết lại gì. Bộ tách khung Rust ở mục 5 được biên dịch thành `libak_ffi.a` và link vào app C của ak-mcu-base
(tuỳ chọn `AK_RUST_PROTO`). Phía C gọi nó qua `extern "C"`; bootloader vẫn dùng bản C. Thư viện này còn mang bộ tách
yêu cầu Modbus (mục 9).

```mermaid
flowchart LR
    ISR["Ngắt UART (C)<br/>đẩy byte vào ring"] --> POLL["task_poll_console (C)"]
    POLL --> FEED["fw_proto_feed(b, ms) (C)"]
    FEED --> RS["ak_rs_proto_feed() (Rust)<br/>BoTach::nap()"]
    RS -- "0: không thuộc khung" --> SH["Shell (C)"]
    RS -- "1: thuộc khung, chưa đủ" --> W["Giữ byte, chờ byte sau"]
    RS -- "2: đủ khung, CRC đúng" --> HF["handle_frame() (C)<br/>INFO, ghi ảnh, CONFIRM..."]
    RS -. "panic" .-> P["ak_rust_panic() → FATAL(RS)<br/>→ nhật ký sự cố trong EEPROM"]
```

Chi phí: +312 B flash, +272 B RAM trên bản demo ([rust-va-c.md mục 2.2](rust-va-c.md#22-cùng-một-module-bộ-tách-khung-fw_proto)).
Mã: [`ak-ffi/src/lib.rs`](../ak-ffi/src/lib.rs), [`ak-ffi/include/ak_rs.h`](../ak-ffi/include/ak_rs.h); phía C:
`src/services/fw/fw_proto.c` của ak-mcu-base.

## 8. Đọc vượt mảng: C chạy tiếp, Rust dừng đúng dòng

```mermaid
flowchart LR
    subgraph C["C"]
        direction TB
        C1["int diem[5]; x = diem[5];"] --> C2["Đọc 4 byte ngay sau mảng"]
        C2 --> C3["Chạy tiếp với giá trị rác<br/>(ghi vượt thì hỏng biến bên cạnh, không ai biết)"]
    end
    subgraph R["Rust (bài 5)"]
        direction TB
        R1["let x = diem[chi_so]; // chi_so = 5"] --> R2["Kiểm chỉ số: 5 ≥ len 5"]
        R2 --> R3["panic handler của ak-kit: tắt ngắt"]
        R3 --> R4["Cổng COM: panic ... tai main.rs:34"]
        R3 --> R5["OLED: PANIC / main.rs dong 34"]
        R3 --> R6["LED nháy nhanh mãi"]
    end
    C ~~~ R
```

Mã: bài 5 [`an-toan/src/main.rs`](../an-toan/src/main.rs), `panic` trong [`ak-kit/src/lib.rs`](../ak-kit/src/lib.rs).

## 9. Slave Modbus RTU với bộ tách yêu cầu Rust (`ak-modbus`)

Tuỳ chọn `AK_RUST_MODBUS` của ak-mcu-base: `mb_slave_poll()` đưa cả khung yêu cầu cho Rust thay vì gọi
`nmbs_server_poll()` của nanoMODBUS. Rust kiểm, C đọc/ghi thanh ghi, Rust dựng câu trả lời.

```mermaid
flowchart TD
    BUS["RS485: byte vào ring (ngắt, C)"] --> IDLE{"Đường dây im ≥ 3,5 ký tự<br/>và không đang gửi?"}
    IDLE -- "chưa" --> BUS
    IDLE -- "rồi" --> TACH["ak_rs_mb_tach() (Rust)<br/>địa chỉ → mã hàm → độ dài → CRC → số lượng"]
    TACH -- "chưa đủ byte" --> CHO{"Im quá 20 ms?"}
    CHO -- "chưa" --> BUS
    CHO -- "rồi" --> BO["Bỏ khung"]
    TACH -- "slave khác, sai CRC, thừa byte" --> BO
    TACH -- "ngoại lệ (01, 02, 03)" --> QB{"Quảng bá?"}
    TACH -- "yêu cầu 03, 04, 06, 16" --> CB["Callback thanh ghi, OTA (C)<br/>cb_read_holding, cb_write_multi..."]
    CB --> QB
    QB -- "có" --> IM["Không trả lời"]
    QB -- "không" --> TL["ak_rs_mb_tra_loi_*() / ak_rs_mb_ngoai_le() (Rust)<br/>dựng câu trả lời + CRC"]
    TL --> GUI["hal_rs485_send() (C, gửi bằng ngắt)"]
```

Mã: [`ak-modbus/src/lib.rs`](../ak-modbus/src/lib.rs), [`ak-ffi/src/lib.rs`](../ak-ffi/src/lib.rs); phía C:
`src/services/modbus/mb_slave.c` của ak-mcu-base. Số đo và phép so với nanoMODBUS:
[rust-va-c.md mục 2.3](rust-va-c.md#23-module-thứ-hai-tách-yêu-cầu-modbus-rtu-thay-nanomodbus).
