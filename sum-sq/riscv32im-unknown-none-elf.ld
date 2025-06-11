MEMORY {
  program (rwx) : ORIGIN = 0x80000000, LENGTH = 10485760
}

ENTRY(_start)

SECTIONS {
  .text.boot : {
    *(.text.boot)
  } > program

  .text : {
    *(.text)
  } > program

  .data : {
    *(.data)
  } > program

  .bss : {
    *(.bss)
  } > program

  .tohost : {
    KEEP(*(.tohost))
  } > program

  .fromhost : {
    KEEP(*(.fromhost))
  } > program

  . = ALIGN(8);
  . = . + 4096;
  _STACK_PTR = .;
  
  . = ALIGN(8);
  _HEAP_PTR = .;
}