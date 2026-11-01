/* Sinh tests/crash_log_c.bin bang chinh crash_log.c cua ak-mcu-base, de test cua
 * ak-boot/src/su_co.rs kiem Rust doc/ghi dung tung byte nhu ban C. Chay tu ak-mcu-base:
 *
 *   gcc -std=c99 -Wall -Wextra -Werror -Isrc/services/sys -Isrc/hal -Isrc/common -Isrc/kernel/inc \
 *       -o /tmp/golden ../ak-mcu-rust/ak-boot/tests/crash_log_c.c src/services/sys/crash_log.c src/common/crc.c
 *   /tmp/golden ../ak-mcu-rust/ak-boot/tests/crash_log_c.bin
 */
#include <stdio.h>
#include <string.h>
#include "crash_log.h"
#include "hal.h"

static uint8_t nvm[256];
int hal_nvm_read(uint32_t off, void* buf, uint32_t len) { memcpy(buf, nvm + off, len); return 0; }
int hal_nvm_write(uint32_t off, const void* buf, uint32_t len) { memcpy(nvm + off, buf, len); return 0; }
void hal_last_dispatch(uint8_t* t, uint8_t* s) { *t = 0xEF; *s = 0; }
uint8_t hal_crash_take(hal_crash_t* c) { (void)c; return 0; }
uint8_t hal_reset_reason(void) { return 0; }

int main(int argc, char** argv) {
	if (argc < 2) { return 2; }
	memset(nvm, 0xFF, sizeof(nvm));
	/* 10 records into 8 slots: the ring wraps, the 2 oldest are gone */
	for (uint32_t i = 1; i <= 10; i++) {
		crash_rec_t r;
		memset(&r, 0, sizeof(r));
		r.kind = (uint8_t)(1 + i % 4);
		r.code = (uint8_t)(0x10 + i);
		r.task = (uint8_t)(i == 10 ? 0xEF : i);
		r.pc = 0x08003100u + i * 0x111u;
		r.lr = 0x0800FFF0u + i;
		memcpy(&r.info, "RUST", 4);
		if (i % 4 == 0) { r.info = 0x00008200u + i; }
		r.sig = (uint8_t)(10 + i);
		if (crash_log_add(&r) != 0) { return 1; }
	}
	printf("count %u\n", crash_log_count());
	FILE* f = fopen(argv[1], "wb");
	fwrite(nvm, 1, 64 + 8 * 20, f);
	fclose(f);
	return 0;
}
