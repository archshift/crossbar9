.extern create_stack

.global userspace_jmp
.type userspace_jmp, %function
.func userspace_jmp
@ Args: [entry, argv, envp]
userspace_jmp:
    msr cpsr_c, #0b11010000
    svc 1
    bx lr
.endfunc
