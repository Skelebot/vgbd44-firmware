# Vagabond 44 keyboard firmware

**This firmware is a work-in-progress**
The `main` branch is guaranteed to be a working and tested on hardware version, other branches *are not*.

## Compiling
Prerequisites:
 - rust nightly (at least 1.57.0) with thumbv6m-none-eabi target
 - GNU binutils-arm-none-eabi
 - GNU make 
 - dfu-util
 
 Optional:
  - xargo (for a much smaller binary size, required for 16K-flash MCUs (see notes))
  
To program the keyboard:
 - compile the binary by running `make build` (or `make build-xargo` for xargo)
 - enter bootloader by holding the 'BOOT' button and tapping the 'RESET' button, then releasing 'BOOT'
 - upload the binary by running `make flash`
 
## Notes
[The BOM](https://github.com/Skelebot/vgbd44#parts-list-bom) lists the part number STMF042**K6T6** for the MCU. This is not the *only* part that can be used - there are pin-compatible MCUs, but they may not have the same amount of flash/ram/etc.
[As seen here](https://www.st.com/en/microcontrollers-microprocessors/stm32f0x2.html) the series that work are STM32F042**K6** (**32K** of FLASH) and STM32F04**K4** (**16K** of FLASH). Because of the global semiconductor shortage, I was able to acquire only 4 of the 32K MCUs, and then the only available to me were the 16K MCUs.

The issue is that that the firmware, when compiled with normal cargo takes up an average of 16-18K of FLASH, which obviously does not fit on the 16K model, but when compiled with `xargo` it takes about 14-15.5K. The 16K FLASH MCUs are usable, but keep in mind that there may not be enough space for any "extra" features (like LEDs), and some very complicated layouts (6-7 or more layers) may not fit by themselves.
