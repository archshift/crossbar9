.section ".init"
.align 4
.arm

.global start

.extern main
.extern bss_start
.extern bss_end

.extern init_interrupts
.extern enable_interrupts
.extern wait_for_interrupt

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

setup_modes:
msr cpsr_c, #0b11010011
    ldr sp, =0x08002000
    ldr lr, =0xFFFF0000
    msr spsr_cxsf, #0

    msr cpsr_c, #0b11010010
    ldr sp, =0x08002800
    ldr lr, =0xFFFF0000
    msr spsr_cxsf, #0

    msr cpsr_c, #0b11010001
    ldr sp, =0xFFFF0000
    ldr lr, =0xFFFF0000
    msr spsr_cxsf, #0

    msr cpsr_c, #0b11010111
    ldr sp, =0x08002800
    ldr lr, =0xFFFF0000
    msr spsr_cxsf, #0

    msr cpsr_c, #0b11011011
    ldr sp, =0x08002800
    ldr lr, =0xFFFF0000
    msr spsr_cxsf, #0

    msr cpsr_c, #0b11011111
    ldr sp, =0x08004000
    ldr lr, =0xFFFF0000

run:
	blx init_interrupts
	bl enable_interrupts

	blx main
end:
	bl wait_for_interrupt
	b end

.ltorg
