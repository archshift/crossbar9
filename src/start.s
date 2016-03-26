.section ".init"
.align 4
.arm

.global start

.extern main
.extern bss_start
.extern bss_end

_brahma_init:
	b start
@ This is just a placeholder, and it will be replaced
@ with the real address by the brahma loader
ret_addr: .word 0xFFFF0000

start:
	@ Clear BSS
	mov r0, #0
	ldr r1, =bss_start
	ldr r2, =bss_end
clr_bss_loop:
	cmp r1, r2
	strlo r0, [r1], #4
	blo clr_bss_loop

	blx main
end:
	b end

.ltorg
