/* Linker script for the STM32F042x4 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 16K
    RAM : ORIGIN = 0x20000000, LENGTH = 6K
}

/* Modified link.x from cortex-m-rt in order to fix an error with PreResetTrampoline */
/*
SECTIONS
{
    .text _stext :
    {
        __stext = .;
        *(.Reset);
        *(.PreResetTrampoline);

        *(.text .text.*);

        *(.HardFaultTrampoline);
        *(.HardFault.*);

        . = ALIGN(4);
        __etext = .;
    } > FLASH
}
*/