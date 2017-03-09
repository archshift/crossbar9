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
.extern handle_und
.extern handle_pre
.extern handle_dta

.type wrap_handle_irq, %function
.func wrap_handle_irq
wrap_handle_irq:
    sub lr, lr, #4
    stmfd sp!, {r0-r3, r12, lr}
    mrs r14, cpsr
    stmfd sp!, {r14}

    blx handle_irq

    ldmfd sp!, {r14}
    msr cpsr_cxsf, r14
    ldmfd sp!, {r0-r3, r12, pc}^
.endfunc

.type wrap_handle_fiq, %function
.func wrap_handle_fiq
wrap_handle_fiq:
    subs pc, lr, #4
.endfunc

.type wrap_handle_swi, %function
.func wrap_handle_swi
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
.endfunc

.type wrap_handle_und, %function
.func wrap_handle_und
wrap_handle_und:
    stmfd sp!, {r0-r3, r12, lr}
    sub r0, lr, #4 @@ addr = LR - 4
    blx handle_und
    ldmfd sp!, {r0-r3, r12, pc}^
    movs pc, lr
.endfunc

.type wrap_handle_pre, %function
.func wrap_handle_pre
wrap_handle_pre:
    blx handle_pre
    subs pc, lr, #4
.endfunc

.type wrap_handle_dta, %function
.func wrap_handle_dta
wrap_handle_dta:
    stmfd sp!, {r0-r3, r12, lr}
    sub r0, lr, #8 @@ addr = LR - 4
    blx handle_dta
    ldmfd sp!, {r0-r3, r12, pc}^
    subs pc, lr, #8
.endfunc

.type enable_interrupts, %function
.func enable_interrupts
enable_interrupts:
    mrs r0, cpsr
    bic r0, r0, #((1 << 6) | (1 << 7))
    msr cpsr_c, r0
    bx lr
.endfunc

.type disable_interrupts, %function
.func disable_interrupts
disable_interrupts:
    mrs r0, cpsr
    orr r0, r0, #((1 << 6) | (1 << 7))
    msr cpsr_c, r0

    ands r0, r0, #((1 << 6) | (1 << 7))
    movne r0, #1 @@ Interrupts were enabled
    moveq r0, #0 @@ Interrupts were disabled
    bx lr
.endfunc

.type wait_for_interrupt, %function
.func wait_for_interrupt
wait_for_interrupt:
    mov r0, #0
    mcr p15, 0, r0, c7, c0, 4
    bx lr
.endfunc

ldr_pc_pc_neg4:
    ldr pc, [pc, #-4]
