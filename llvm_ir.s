	.text
	.file	"llvm_ir"
	.section	.rodata.cst4,"aM",@progbits,4
	.p2align	2               # -- Begin function main
.LCPI0_0:
	.long	1077936128              # float 3
	.text
	.globl	main
	.p2align	4, 0x90
	.type	main,@function
main:                                   # @main
	.cfi_startproc
# %bb.0:                                # %entry
	movl	$1103101952, -16(%rsp)  # imm = 0x41C00000
	movl	$1119879168, -20(%rsp)  # imm = 0x42C00000
	movl	$1113063424, -24(%rsp)  # imm = 0x42580000
	movl	$1113063424, -12(%rsp)  # imm = 0x42580000
	movss	.LCPI0_0(%rip), %xmm0   # xmm0 = mem[0],zero,zero,zero
	retq
.Lfunc_end0:
	.size	main, .Lfunc_end0-main
	.cfi_endproc
                                        # -- End function
	.section	".note.GNU-stack","",@progbits
