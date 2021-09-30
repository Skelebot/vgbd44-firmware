TARGET_DIR=target/thumbv6m-none-eabi/release
BIN_NAME=firmware

all-xargo: build-xargo objcopy flash
all: build objcopy flash

.PHONY: build build-xargo objcopy flash size clean all all-xargo

build:
	cargo build --release

build-xargo:
	xargo build --release

objcopy:
	arm-none-eabi-objcopy -O binary $(TARGET_DIR)/$(BIN_NAME) $(TARGET_DIR)/$(BIN_NAME).bin

flash: objcopy
	sudo dfu-util -a 0 -s 0x08000000:leave -D $(TARGET_DIR)/$(BIN_NAME).bin
	
size:
	arm-none-eabi-size $(TARGET_DIR)/$(BIN_NAME)

clean:
	cargo clean