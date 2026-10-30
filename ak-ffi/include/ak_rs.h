/**
 ******************************************************************************
 * @brief:  Bo tach khung fw_proto viet bang Rust (repo ak-mcu-rust, crate ak-ffi),
 *          link vao firmware C cua ak-mcu-base khi build voi AK_RUST_PROTO.
 *          Phai khop voi ak-ffi/src/lib.rs.
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

/* Phia C cung cap: Rust panic -> FATAL("RS", ...) -> nhat ky su co. */
extern void ak_rust_panic(void) __attribute__((noreturn));

#ifdef __cplusplus
}
#endif

#endif /* __AK_RS_H__ */
