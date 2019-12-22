@ Thumbwrestler
@ mic, 2005-2006 | micol972@gmail.com
@
@ Runs THUMB instructions and checks for valid results. Useful for people developing THUMB emulators
@ Compile with devkitarm

.extern tw_draw_text
.extern tw_draw_result_

.text
.thumb

.equ C_MASK,	0x01
.equ C_SHIFT,	0
.equ V_MASK,	0x02
.equ V_SHIFT,	1
.equ Rd_MASK,	0x100
.equ Rd_SHIFT,	8


.macro TEST_Rd reg val
	cmp \reg,\val
	beq 1f
	lsl r4,r5,#Rd_SHIFT
	orr r6,r4
	1:
.endm

.type tw_draw_result, %function
.func tw_draw_result
tw_draw_result:
	mov		r1,r6
	push	{lr}
	bl		tw_draw_result_
	pop 	{pc}
.endfunc

.global tw_test0
.type tw_test0, %function
.func tw_test0
.align 1
tw_test0:
	push 	{lr}

	ldr 	r0,=_szALU
	mov 	r1,#88
	mov 	r2,#1
	mov 	r3,#4
	bl 	tw_draw_text

	@ ADD
	mov 	r6,#0
	mov 	r0,#0
	mov 	r1,#1
	mov 	r2,#4
	mov 	r3,#5
	add 	r0,r1,#2
	TEST_Rd r0,#3
	add 	r0,r1,r2
	TEST_Rd r0,#5
	mov 	r0,#2
	mov 	r1,#0
	mov 	r2,#0
	add 	r1,r1,#0	@ clear carry
	add 	r0,r11
	bcc 	_add_ok_1
	orr 	r6,r5
	_add_ok_1:
	TEST_Rd r0,#1
	mov 	r0,#0
	add 	r0,pc,#4
	beq 	_add_ok_2
	lsl 	r4,r5,#3
	_add_labelpc:
	orr 	r6,r4
	_add_ok_2:
	ldr 	r1,=_add_labelpc
	TEST_Rd r0,r1

	ldr 	r0,=0xFFFFFF01
	mov 	r1,r0
	add 	r0,#0xFF
	beq 	_add_ok_3
	lsl 	r4,r5,#3
	orr 	r6,r4
	_add_ok_3:
	TEST_Rd r0,#0
	add 	r1,#0xFF
	bcs 	_add_ok_4
	orr 	r6,r5
	_add_ok_4:
	ldr 	r0,=_szADD
	blx 	tw_draw_result
	add 	r7,#8


	@ ASR
	mov 	r6,#0

	@ Test ASR by imm==32
	ldr 	r0,=0x80000000
	ldr 	r1,=0xFFFFFFFF
	mov 	r2,#0
	add 	r2,r2,#0	@ clear carry
	asr 	r0,r0,#32
	bcs 	_asr_ok_1
	orr 	r6,r5
	_asr_ok_1:
	TEST_Rd r1,r0

	ldr 	r0,=0x80000000
	asr 	r0,r0,#32
	bmi 	_asr_ok_3
	lsl 	r4,r5,#1
	orr 	r6,r4
	_asr_ok_3:
	ldr 	r0,=0x80000000
	asr 	r0,r0,#32
	bne 	_asr_ok_4
	lsl 	r4,r5,#3
	orr 	r6,r4
	_asr_ok_4:
	ldr 	r0,=_szASR
	blx 	tw_draw_result
	add 	r7,#8

	@ BIC
	mov 	r1,#0
	add 	r2,r2,#0	@ clear carry
	ldr 	r2,=0xFFFFFFFF
	ldr 	r3,=0xC000000D
	bic 	r2,r3
	bcc		_bic_cc
	bmi		_bic_mi
	beq		_bic_eq

	_bic_cc:
	mov		r3,#1
	orr 	r1,r3
	b		_bic_fin

	_bic_mi:
	mov		r3,#2
	orr 	r1,r3
	b		_bic_fin

	_bic_eq:
	mov		r3,#8
	orr 	r1,r3
	b		_bic_fin

	_bic_fin:
	ldr 	r3,=0x1FFFFFF9
	cmp 	r2,r3
	bne		_bic_ne
	b		_bic_fin2

	_bic_ne:
	mov		r3,#16
	orr 	r1,r3
	b		_bic_fin2

	_bic_fin2:
	ldr 	r0,=_szBIC
	blx 	tw_draw_result
	add 	r7,#8

	@ CMP
	mov 	r6,#0

	mov 	r0, #1
	neg 	r1,r0
	cmp		r1,#0
	blt     cmp_end
	mov 	r6,#V_MASK

cmp_end:
	ldr 	r0,=_szCMP
	blx 	tw_draw_result

	add 	r7,#8

	@ LSL
	mov 	r6,#0

	@ Test LSL by reg==32
	mov 	r0,#3
	mov 	r1,#32
	lsl 	r0,r1
	bcs 	_lsl_ok_1
	orr 	r6,r5
	_lsl_ok_1:
	cmp 	r0,#0
	beq 	_lsl_ok_2
	lsl 	r4,r5,#Rd_SHIFT
	orr 	r6,r4
	_lsl_ok_2:
	mov 	r0,#3
	lsl 	r0,r1
	bpl 	_lsl_ok_3
	lsl 	r4,r5,#1
	orr 	r6,r4
	_lsl_ok_3:
	mov 	r0,#3
	lsl 	r0,r1
	beq 	_lsl_ok_4
	lsl 	r4,r5,#3
	orr 	r6,r4
	_lsl_ok_4:
	ldr 	r0,=_szLSL
	blx 	tw_draw_result
	add 	r7,#8


	@ LSR
	mov 	r6,#0

	@ Test LSR by imm==32
	ldr 	r0,=0x80000000
	mov 	r1,#2
	mov 	r2,#0

	add 	r2,r2,#0	@ clear carry
	lsr 	r1,r0,#32
	bcs 	_lsr_ok_1
	orr 	r6,r5
	_lsr_ok_1:
	cmp 	r1,#0
	beq 	_lsr_ok_2
	lsl 	r4,r5,#Rd_SHIFT
	orr 	r6,r4
	_lsr_ok_2:
	ldr 	r0,=_szLSR
	blx 	tw_draw_result
	add 	r7,#8


	@ MUL
	mov 	r6,#0

	mov 	r0,#1
	mov 	r1,#20
	ldr 	r2,=0xFFFFFFF6
	ldr 	r3,=0xFFFFFF38

	lsr 	r0,r0,#1	@ set carry
	mul 	r1,r2
	@bcs 	_mul_ok_1
	@orr 	r6,r5
	@_mul_ok_1:
	TEST_Rd r1,r3
	mov 	r1,#20
	mul 	r1,r2
	bmi 	_mul_ok_2
	lsl 	r4,r5,#1
	orr 	r6,r4
	_mul_ok_2:
	mov 	r1,#20
	mul 	r1,r2
	bne 	_mul_ok_3
	lsl 	r4,r5,#3
	orr 	r6,r4
	_mul_ok_3:
	ldr 	r0,=_szMUL
	blx 	tw_draw_result
	add 	r7,#8


	@ MVN
	mov 	r6,#0

	ldr 	r0,=0xFFFFFF00
	mvn 	r0,r0
	bpl 	_mvn_ok_1
	lsl 	r4,r5,#1
	orr 	r6,r4
	_mvn_ok_1:
	TEST_Rd r0,#0xFF
	ldr 	r0,=_szMVN
	blx 	tw_draw_result
	add 	r7,#8


	@ NEG
	mov 	r6,#0

	mov 	r0,#1
	neg 	r1,r0
	bne 	_neg_ok_1
	lsl 	r4,r5,#3
	orr 	r6,r4
	_neg_ok_1:
	neg 	r1,r0
	bmi 	_neg_ok_2
	lsl 	r4,r5,#1
	orr 	r6,r4
	_neg_ok_2:
	neg 	r1,r1
	bpl 	_neg_ok_3
	lsl 	r4,r5,#1
	orr 	r6,r4
	_neg_ok_3:
	neg 	r0,r0
	ldr 	r2,=0xFFFFFFFF
	TEST_Rd r0,r2
	ldr 	r0,=_szNEG
	blx 	tw_draw_result
	add 	r7,#8

	@ SUB
	mov 	r6,#0

	mov 	r0, #1
	neg 	r1,r0
	sub		r2,r1,#1
	blt     sub_end
	mov 	r6,#V_MASK

sub_end:
	ldr 	r0,=_szSUB
	blx 	tw_draw_result

	add 	r7,#8

	@ ROR
	mov 	r6,#0

	@ Test ROR by reg==0
	ldr 	r0,=0x80000000
	mov 	r1,r0
	mov 	r2,#1
	mov 	r3,#0
	lsr 	r2,r2,#1	@ set carry
	ror 	r0,r3
	bcs 	_ror_ok_1
	orr 	r6,r5
	_ror_ok_1:
	cmp 	r0,r1
	beq 	_ror_ok_2
	lsl 	r4,r5,#Rd_SHIFT
	orr 	r6,r4
	_ror_ok_2:

	@ Test ROR by reg==16
	ldr 	r0,=0x80000000
	ldr 	r1,=0x8000
	mov 	r2,#1
	mov 	r3,#16
	lsr 	r2,r2,#1	@ set carry
	ror 	r0,r3
	cmp		r0,r1
	bcs 	_ror_ok_3
	orr 	r6,r5
	_ror_ok_3:
	cmp 	r0,r1
	beq 	_ror_ok_4
	lsl 	r4,r5,#Rd_SHIFT
	orr 	r6,r4
	_ror_ok_4:

	@ Test ROR by reg>32
	ldr 	r0,=0x80000000
	mov 	r1,#66
	ror 	r0,r1
	bcc 	_ror_ok_5

	orr 	r6,r5
	_ror_ok_5:
	ldr 	r2,=0x20000000
	cmp 	r0,r2
	beq 	_ror_ok_6
	lsl 	r4,r5,#Rd_SHIFT
	orr 	r6,r4
	_ror_ok_6:

	@ Test ROR by reg==32
	ldr 	r0,=0x80000000
	mov 	r2,r0
	mov 	r1,#32
	ror 	r0,r1
	bcs 	_ror_ok_7
	orr 	r6,r5
	_ror_ok_7:
	ldr 	r0,=0x80000000
	ror 	r0,r1
	bmi 	_ror_ok_8
	lsl 	r4,r5,#1
	orr 	r6,r4
	_ror_ok_8:
	ldr 	r0,=0x80000000
	ror 	r0,r1
	bne 	_ror_ok_9
	lsl 	r4,r5,#3
	orr 	r6,r4
	_ror_ok_9:
	cmp 	r0,r2
	beq 	_ror_ok_10
	lsl 	r4,r5,#Rd_SHIFT
	orr 	r6,r4
	_ror_ok_10:
	ldr 	r0,=_szROR
	blx 	tw_draw_result
	add 	r7,#8

	pop {pc}
.endfunc
.pool
.align

.global tw_test1
.type tw_test1, %function
.func tw_test1
tw_test1:
	push 	{lr}

	ldr 	r0,=_szLDR1
	mov 	r1,#64
	mov 	r2,#1
	mov 	r3,#4
	bl 	tw_draw_text

	@ LDR Rd,[Rb,#imm]
	mov 	r6,#0
	ldr 	r0,=romvar2-2
	ldr 	r1,[r0,#4]
	ldr 	r2,=0x8f00ff00
	TEST_Rd r1,r2
	ldr 	r0,=_szLDR
	blx 	tw_draw_result
	add 	r7,#8

	@ LDR Rd,[Rb,Ro]
	mov 	r6,#0
	ldr 	r0,=romvar2
	ldr 	r2,=0x8f00ff00
	mov 	r3,#2
	ldr 	r1,[r0,r3]
	TEST_Rd r1,r2
	ldr 	r0,=_szLDR
	blx 	tw_draw_result
	add 	r7,#8

	@ LDRB Rd,[Rb,Ro]
	mov 	r6,#0
	ldr 	r0,=romvar2
	ldr 	r2,=0x8f
	mov 	r3,#1
	ldrb 	r1,[r0,r3]
	TEST_Rd r1,r2
	ldr 	r0,=_szLDRB
	blx 	tw_draw_result
	add 	r7,#8

	@ LDRH Rd,[Rb,Ro]
	mov 	r6,#0
	ldr 	r0,=romvar2
	ldr 	r2,=0x8f00
	mov 	r3,#0
	ldrh 	r1,[r0,r3]
	TEST_Rd r1,r2
	ldr 	r0,=_szLDRH1
	blx 	tw_draw_result
	add 	r7,#8

	@ LDRH Rd,[Rb,#nn]
	mov 	r6,#0
	ldr 	r0,=romvar2
	ldr 	r2,=0x8f00
	ldrh 	r1,[r0,#0]
	TEST_Rd r1,r2
	ldr 	r0,=_szLDRH2
	blx 	tw_draw_result
	add 	r7,#8

	@ LDRSH Rd,[Rb,Ro]
	mov 	r6,#0
	ldr 	r0,=romvar2
	ldr 	r2,=0xFFFF8F00
	mov 	r3,#0
	ldsh 	r1,[r0,r3]
	TEST_Rd r1,r2
	ldr 	r0,=_szLDRSH
	blx 	tw_draw_result
	add 	r7,#8

	@ LDRSB Rd,[Rb,Ro]
	mov 	r6,#0
	ldr 	r0,=romvar2
	ldr 	r2,=0xFFFFFF8F
	mov 	r3,#1
	ldsb 	r1,[r0,r3]
	TEST_Rd r1,r2
	ldr 	r0,=_szLDRSB
	blx 	tw_draw_result
	add 	r7,#8

	pop 	{pc}
.endfunc
.pool
.align 1

.global tw_test2
.type tw_test2, %function
.func tw_test2
tw_test2:
	push 	{lr}

	ldr 	r0,=_szLDM1
	mov 	r1,#72
	mov 	r2,#1
	mov 	r3,#4
	bl 	tw_draw_text


	@ LDMIA Rb!,{Rlist}
	mov 	r6,#0 @clear flags
	mov 	r1,#0
	ldr 	r3,=_var64
	sub 	r3,r3,#4
	ldmia 	r3!,{r1,r2}
	ldr 	r0,=_var64+4
	cmp 	r3,r0
	beq		_ldm_r2_r0
	b		_ldm_done

	_ldm_r2_r0:
	ldr 	r0,=_var64
	ldr		r0,[r0]
	cmp 	r1,r0
	beq		_ldm_r1_r0
	b		_ldm_done

	_ldm_r1_r0:
	ldr 	r0,=_var64+4
	ldr		r0,[r0]
	cmp 	r2,r0

	_ldm_done:
	ldr 	r0,=_szLDMIA
	blx 	tw_draw_result
	add 	r7,#8

	@ STMIA Rb!,{Rlist}
	mov 	r6,#0 @clear flags
	ldr 	r1,=0x44332211
	ldr 	r2,=0x88776655
	ldr 	r3,=_tvar64
	sub 	r3,r3,#4
	stmia 	r3!,{r1,r2}
	ldr 	r0,=_tvar64+4
	cmp 	r3,r0
	beq		_stm_r2_r0
	b		_stm_done

	_stm_r2_r0:
	ldr 	r0,=_tvar64
	ldr		r0,[r0]
	cmp 	r1,r0
	beq		_stm_r1_r0
	b		_stm_done

	_stm_r1_r0:
	ldr 	r0,=_tvar64+4
	ldr		r0,[r0]
	cmp 	r2,r0

	_stm_done:
	ldr 	r0,=_szSTMIA
	blx 	tw_draw_result
	add 	r7,#8

	pop 	{pc}
.endfunc


.align 3
_var64:		.word 0x11223344,0x55667788
_tvar64:	.word 0x11223344,0x55667788


.align 2

_szALU:		.asciz "ALU TEST"
_szLDR1:	.asciz "LDR/STR TEST 1"
_szLDM1:	.asciz "LDM/STM TEST"

_szADC:		.asciz "ADC"
_szADD:		.asciz "ADD"
_szAND:		.asciz "AND"
_szASR:		.asciz "ASR"
_szBIC:		.asciz "BIC"
_szCMP:		.asciz "CMP"
_szLSL:		.asciz "LSL"
_szLSR:		.asciz "LSR"
_szMUL:		.asciz "MUL"
_szMVN:		.asciz "MVN"
_szNEG:		.asciz "NEG"
_szORR:		.asciz "ORR"
_szROR:		.asciz "ROR"
_szSUB:		.asciz "SUB"

_szLDR:		.asciz "LDR"
_szLDRH:	.asciz "LDRH"
_szLDRH1:	.asciz "LDRH R"
_szLDRH2:	.asciz "LDRH \\"
_szLDRB:	.asciz "LDRB"
_szLDRSH:	.asciz "LDRSH"
_szLDRSB:	.asciz "LDRSB"
_szLDM:		.asciz "LDM"
_szLDMIA:	.asciz "LDMIA"
_szSTR:		.asciz "STR"
_szSTRH:	.asciz "STRH"
_szSTRB:	.asciz "STRB"
_szSTMIA:	.asciz "STMIA"

_szOK:		.asciz "OK"
_szBad:		.asciz "BAD"
_szRd:		.asciz "R^"
_szRn:		.asciz "R_"
_szC:		.asciz "C"
_szN:		.asciz "N"
_szV:		.asciz "V"
_szZ:		.asciz "Z"
_szGT:		.asciz "+"
_szGE:		.asciz ","
_szLT:		.asciz "-"
_szLE:		.asciz "."

.align 2
.end
