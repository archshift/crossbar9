.section ".text"
.align 4
.arm

.global wrap_handle_irq
.global wrap_handle_fiq
.global wrap_handle_swi
.global wrap_handle_und
.global wrap_handle_pre
.global wrap_handle_dta
.global disable_interrupts
.global enable_interrupts
.global wait_for_interrupt
.global ldr_pc_pc_neg4

.extern panic_3ds
.extern handle_irq
.extern handle_swi

wrap_handle_irq:
    sub lr, lr, #4
    stmfd sp!, {r0-r3, r12, lr}
    mrs r14, cpsr
    stmfd sp!, {r14}

    blx handle_irq

    ldmfd sp!, {r14}
    msr cpsr_cxsf, r14
    ldmfd sp!, {r0-r3, r12, pc}^

wrap_handle_fiq:
    subs pc, lr, #4

wrap_handle_swi:
    stmfd sp!, {r0-r3, r12, lr}

    mrs r0, spsr
    tst r0, #(1 << 5)

    @ if CPU in thumb state
    ldrneh r0, [lr,#-2]
    bicne r0, r0, #0xFF00
    @ else
    ldreq r0, [lr,#-4]
    biceq r0, r0, #0xFF000000

    blx handle_swi
    ldmfd sp!, {r0-r3, r12, pc}^

wrap_handle_und:
    blx panic_3ds
    movs pc, lr

wrap_handle_pre:
    blx panic_3ds
    subs pc, lr, #4

wrap_handle_dta:
    blx panic_3ds
    subs pc, lr, #8

enable_interrupts:
    mrs r0, cpsr
    bic r0, r0, #((1 << 6) | (1 << 7))
    msr cpsr_c, r0
    bx lr

disable_interrupts:
    mrs r0, cpsr
    orr r0, r0, #((1 << 6) | (1 << 7))
    msr cpsr_c, r0
    bx lr

wait_for_interrupt:
    mcr p15, 0, r0, c7, c0, 4
    bx lr

ldr_pc_pc_neg4:
    ldr pc, [pc, #-4]