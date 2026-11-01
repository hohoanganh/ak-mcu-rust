/**
 ******************************************************************************
 * @brief:  Bo tach khung viet bang Rust (repo ak-mcu-rust, crate ak-ffi), link vao
 *          firmware C cua ak-mcu-base: fw_proto (AK_RUST_PROTO) va yeu cau Modbus RTU
 *          phia slave (AK_RUST_MODBUS). Phai khop voi ak-ffi/src/lib.rs.
 ******************************************************************************
**/

#ifndef __AK_RS_H__
#define __AK_RS_H__

#ifdef __cplusplus
extern "C"
{
#endif

#include <stdint.h>

#define AK_RS_DU_LIEU_TOI_DA	(132)	/* = FW_PROTO_MAX_PAYLOAD */

typedef struct {
	uint8_t  cmd;
	uint8_t  seq;
	uint16_t len;
	uint8_t  du_lieu[AK_RS_DU_LIEU_TOI_DA];
} ak_rs_khung_t;

/* Bo khung dang do (fw_proto_init). */
extern void ak_rs_proto_reset(void);

/* 1 neu dang o giua mot khung (fw_proto_busy). */
extern uint8_t ak_rs_proto_busy(void);

/* Nap mot byte. 0: byte khong thuoc khung (cua shell); 1: thuoc khung;
 * 2: thuoc khung va vua nhan du mot khung dung CRC, da chep vao *ra. */
extern uint8_t ak_rs_proto_feed(uint8_t b, uint32_t now_ms, ak_rs_khung_t* ra);

/*---------------------------------------------------------------- Modbus RTU (slave) */

#define AK_RS_MB_GHI_TOI_DA		(123)	/* thanh ghi toi da mot lan ghi nhieu (16) */

#define AK_RS_MB_THIEU_BYTE		(0)		/* khung chua du: cho them byte */
#define AK_RS_MB_BO_QUA			(1)		/* slave khac, sai CRC, sai do dai: khong tra loi */
#define AK_RS_MB_NGOAI_LE		(2)		/* tra loi ma ngoai le (tru quang ba) */
#define AK_RS_MB_YEU_CAU		(3)		/* yeu cau hop le (quang ba: lam, khong tra loi) */

typedef struct {
	uint8_t  loai;			/* AK_RS_MB_... */
	uint8_t  ma_ham;		/* 3, 4, 6, 16 (hoac ma ham cua ngoai le) */
	uint8_t  quang_ba;		/* 1: gui toi dia chi 0 */
	uint8_t  ngoai_le;		/* ma ngoai le khi loai = AK_RS_MB_NGOAI_LE */
	uint16_t dia_chi;		/* thanh ghi dau */
	uint16_t so_luong;
	uint16_t gia_tri[AK_RS_MB_GHI_TOI_DA];	/* 6: gia_tri[0]; 16: so_luong gia tri */
} ak_rs_mb_t;

/* Tach mot khung yeu cau (ca khung, sau khi duong day im 3,5 ky tu) gui toi slave
 * dia_chi. Tra ve ra->loai. */
extern uint8_t ak_rs_mb_tach(const uint8_t* khung, uint16_t len, uint8_t dia_chi, ak_rs_mb_t* ra);

/* Dung cau tra loi vao ra[cap], tra ve so byte (0: tham so sai hoac cap thieu). */
extern uint16_t ak_rs_mb_tra_loi_doc(uint8_t dia_chi, uint8_t ma_ham, const uint16_t* thanh_ghi,
									 uint16_t n, uint8_t* ra, uint16_t cap);
extern uint16_t ak_rs_mb_tra_loi_ghi(uint8_t dia_chi, uint8_t ma_ham, uint16_t a, uint16_t b,
									 uint8_t* ra, uint16_t cap);
extern uint16_t ak_rs_mb_ngoai_le(uint8_t dia_chi, uint8_t ma_ham, uint8_t ma, uint8_t* ra, uint16_t cap);

/* Phia C cung cap: Rust panic -> FATAL("RS", ...) -> nhat ky su co. */
extern void ak_rust_panic(void) __attribute__((noreturn));

#ifdef __cplusplus
}
#endif

#endif /* __AK_RS_H__ */
