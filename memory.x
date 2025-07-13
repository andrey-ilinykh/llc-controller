# Memory layout for STM32F103
# The STM32F103 has 64KB Flash and 20KB RAM

MEMORY
{
 /* 64 KB Flash starting at 0x08000000 */
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K

  /* 20 KB RAM starting at 0x20000000 */
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}
