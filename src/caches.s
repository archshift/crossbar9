.global add_mpu_regions
.type add_mpu_regions, %function
.func add_mpu_regions
add_mpu_regions:
    push {r0-r7,lr}
    ldmia r0, {r0-r7}
    mcr p15, 0, r0, c6, c0, 0
    mcr p15, 0, r1, c6, c1, 0
    mcr p15, 0, r2, c6, c2, 0
    mcr p15, 0, r3, c6, c3, 0
    mcr p15, 0, r4, c6, c4, 0
    mcr p15, 0, r5, c6, c5, 0
    mcr p15, 0, r6, c6, c6, 0
    mcr p15, 0, r7, c6, c7, 0
    pop {r0-r7,pc}
.endfunc

.global add_mpu_cachedata
.type add_mpu_cachedata, %function
.func add_mpu_cachedata
add_mpu_cachedata:
    @ r0: cacheability bitset
    @ r1: bufferability bitset
    @ r2: permissions bitset
    mcr p15, 0, r0, c2, c0, 0
    mcr p15, 0, r0, c2, c0, 1
    mcr p15, 0, r1, c3, c0, 0
    mcr p15, 0, r2, c5, c0, 0
    bx lr
.endfunc

.global enable_mpu
.type enable_mpu, %function
.func enable_mpu
enable_mpu:
    @ r0: status to set
    mrc p15, 0, r1, c1, c0, 0
    orr r0, r1, r0,lsl #0
    mcr p15, 0, r0, c1, c0, 0
    mov r0, #0
    mcr p15, 0, r0, c7, c5, 4 @ PrefetchFlush
    bx lr
.endfunc

.global enable_icache
.type enable_icache, %function
.func enable_icache
enable_icache:
    @ r0: status to set
    mrc p15, 0, r1, c1, c0, 0
    orr r0, r1, r0,lsl #12
    mcr p15, 0, r0, c1, c0, 0
    bx lr
.endfunc

.global enable_dcache
.type enable_dcache, %function
.func enable_dcache
enable_dcache:
    @ r0: status to set
    mrc p15, 0, r1, c1, c0, 0
    orr r0, r1, r0,lsl #2
    mcr p15, 0, r0, c1, c0, 0
    bx lr
.endfunc

.global enable_wbuf
.type enable_wbuf, %function
.func enable_wbuf
enable_wbuf:
    @ r0: status to set
    mrc p15, 0, r1, c1, c0, 0
    orr r0, r1, r0,lsl #3
    mcr p15, 0, r0, c1, c0, 0
    bx lr
.endfunc

.global invalidate_icache
.type invalidate_icache, %function
.func invalidate_icache
invalidate_icache:
    mov r0, #0
    mcr p15, 0, r0, c7, c5, 0
    bx lr
.endfunc

.global invalidate_clean_dcache
.type invalidate_clean_dcache, %function
.func invalidate_clean_dcache
invalidate_clean_dcache:
    mov r0, #0
    mcr p15, 0, r0, c7, c14, 0
    bx lr
.endfunc
