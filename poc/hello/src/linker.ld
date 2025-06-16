/* Fixed linker script that creates actual loadable segments */

ENTRY(_start)
SECTIONS {
  /* Text section */
  . = 0x80000000;
  .text : { 
    *(.text*) 
  }
  
    /* Force page alignment for data segment */
  . = ALIGN(0x1000);
  
  /* Data sections - these will create a loadable R+W segment */
  .rodata : { 
    *(.rodata*) 
  }
  
  .data : { 
    *(.data*) 
    *(.sdata*)
  }
  
  .bss : { 
    _bss_start = .;
    *(.bss*) 
    *(.sbss*)
    *(COMMON)
    *(.scommon)
    _bss_end = .;
    
    /* Force allocation of BSS space */
    . = ALIGN(16);
    . = . + 16;  /* Ensure at least 16 bytes allocated */
  }
  
  /* Critical: Reserve stack space in a loadable section */
  .stack : {
    _STACK_SIZE = DEFINED(_STACK_SIZE) ? _STACK_SIZE : 65536;
    . = . + _STACK_SIZE;
    . = ALIGN(16);
    _STACK_PTR = .;
  }
  
  /* Heap space */
  .heap : {
    . = ALIGN(8);
    _HEAP_PTR = .;
    . = . + 1024;  /* Reserve some heap space */
  }
}
