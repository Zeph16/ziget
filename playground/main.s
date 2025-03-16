	.text
	.file	"ziget"
	.section	.rodata.cst8,"aM",@progbits,8
	.p2align	3, 0x0                          # -- Begin function greet_times
.LCPI0_0:
	.quad	0x3ff0000000000000              # double 1
	.text
	.globl	greet_times
	.p2align	4, 0x90
	.type	greet_times,@function
greet_times:                            # @greet_times
	.cfi_startproc
# %bb.0:                                # %entry
	subq	$24, %rsp
	.cfi_def_cfa_offset 32
	movq	%rdi, 16(%rsp)
	movsd	%xmm0, 8(%rsp)
	movsd	8(%rsp), %xmm0                  # xmm0 = mem[0],zero
	movsd	%xmm0, (%rsp)
.LBB0_1:                                # %loop
                                        # =>This Inner Loop Header: Depth=1
	movsd	(%rsp), %xmm0                   # xmm0 = mem[0],zero
	xorps	%xmm1, %xmm1
	movaps	%xmm1, %xmm2
	cmpeqsd	%xmm0, %xmm2
	movq	%xmm2, %rax
                                        # kill: def $eax killed $eax killed $rax
	andl	$1, %eax
                                        # kill: def $al killed $al killed $eax
	ucomisd	%xmm1, %xmm0
	jne	.LBB0_4
	jp	.LBB0_4
	jmp	.LBB0_3
.LBB0_2:                                # %afterloop
	addq	$24, %rsp
	.cfi_def_cfa_offset 8
	retq
.LBB0_3:                                # %then
	.cfi_def_cfa_offset 32
	jmp	.LBB0_2
.LBB0_4:                                # %else
                                        #   in Loop: Header=BB0_1 Depth=1
	jmp	.LBB0_5
.LBB0_5:                                # %merge
                                        #   in Loop: Header=BB0_1 Depth=1
	movq	16(%rsp), %rsi
	leaq	.Lstr(%rip), %rdi
	movb	$0, %al
	callq	printf@PLT
	movsd	(%rsp), %xmm0                   # xmm0 = mem[0],zero
	movsd	.LCPI0_0(%rip), %xmm1           # xmm1 = [1.0E+0,0.0E+0]
	subsd	%xmm1, %xmm0
	movsd	%xmm0, (%rsp)
	jmp	.LBB0_1
.Lfunc_end0:
	.size	greet_times, .Lfunc_end0-greet_times
	.cfi_endproc
                                        # -- End function
	.section	.rodata.cst8,"aM",@progbits,8
	.p2align	3, 0x0                          # -- Begin function main
.LCPI1_0:
	.quad	0x4008000000000000              # double 3
	.text
	.globl	main
	.p2align	4, 0x90
	.type	main,@function
main:                                   # @main
	.cfi_startproc
# %bb.0:                                # %entry
	pushq	%rax
	.cfi_def_cfa_offset 16
	leaq	.Lstr.1(%rip), %rdi
	movsd	.LCPI1_0(%rip), %xmm0           # xmm0 = [3.0E+0,0.0E+0]
	callq	greet_times@PLT
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end1:
	.size	main, .Lfunc_end1-main
	.cfi_endproc
                                        # -- End function
	.type	.Lstr,@object                   # @str
	.section	.rodata.str1.1,"aMS",@progbits,1
.Lstr:
	.asciz	"Hello, %s\n"
	.size	.Lstr, 11

	.type	.Lstr.1,@object                 # @str.1
.Lstr.1:
	.asciz	"Ziget"
	.size	.Lstr.1, 6

	.section	".note.GNU-stack","",@progbits
	.addrsig
	.addrsig_sym printf
	.addrsig_sym greet_times
