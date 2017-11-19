.global cbench_busywait
.type cbench_busywait, %function
.func cbench_busywait
cbench_busywait:
    cmp r0, r1
    addne r0, r0, #1
    bne cbench_busywait
    bx lr
.endfunc