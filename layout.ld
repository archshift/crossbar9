__aeabi_unwind_cpp_pr0 = abort;
__aeabi_unwind_cpp_pr1 = abort;
__aeabi_unwind_cpp_pr2 = abort;
__exidx_start          = abort;
__exidx_end            = abort;

__aeabi_memcpy         = memcpy;
__aeabi_memcpy4        = memcpy;

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

