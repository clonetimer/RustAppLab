/* QEMU lm3s6965evb Cortex-M3 teaching target only.
 * Not a linker configuration for an arbitrary real development board. */
MEMORY
{
    FLASH : ORIGIN = 0x00000000, LENGTH = 256K
    RAM   : ORIGIN = 0x20000000, LENGTH = 64K
}
