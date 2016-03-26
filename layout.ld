rust_begin_unwind      = panic_3ds;
__aeabi_unwind_cpp_pr0 = panic_3ds;
__aeabi_unwind_cpp_pr1 = panic_3ds;
__aeabi_unwind_cpp_pr2 = panic_3ds;
__exidx_start          = panic_3ds;
__exidx_end            = panic_3ds;

ENTRY(start)
SECTIONS
{
	. = 0x23F00000;

	kernel_phys_start = .;

	.init : {
		*(.init)
	}

	.text : {
		*(.text)
		*(.text.*)
	}

	.rodata : {
		*(.rodata*)
	}
	.data : {
		*(.data*)
	}
	.bss : {
		bss_start = .;
		*(.bss*)
		*(COMMON)
		bss_end = .;
	}

	__exidx_start = .;
	.ARM.exidx : {
		*(.ARM.exidx* .gnu.linkonce.armexidx.*)
	}
	__exidx_end = .;

	kernel_phys_end = .;
}

