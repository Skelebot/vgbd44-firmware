TARGET_DIR=target/thumbv6m-none-eabi/release
BIN_NAME=firmware

all: build objcopy flash

build:
	xargo build --release

objcopy:
	arm-none-eabi-objcopy -O binary $(TARGET_DIR)/$(BIN_NAME) $(TARGET_DIR)/$(BIN_NAME).bin

flash: build objcopy
	dfu-util -a 0 -s 0x08000000:leave -D $(TARGET_DIR)/$(BIN_NAME).bin
	
size:
	arm-none-eabi-size $(TARGET_DIR)/$(BIN_NAME)