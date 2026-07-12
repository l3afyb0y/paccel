DRIVERDIR?=`pwd`/driver
MODULEDIR?=/lib/modules/`uname -r`/kernel/drivers/usb

DRIVER_CFLAGS ?= -DFIXEDPT_BITS=$(shell getconf LONG_BIT)

build:
	$(MAKE) DRIVER_CFLAGS="$(DRIVER_CFLAGS)" -C $(DRIVERDIR)

build_debug: override DRIVER_CFLAGS += -g -DDEBUG
build_debug: build

test: 
	$(MAKE) -C $(DRIVERDIR) test
test_debug:
	$(MAKE) -C $(DRIVERDIR) test_debug

dev_cli:
	cargo watch -x 'run'

build_cli:
	cargo build --bin paccel --release

package:
	makepkg --cleanbuild

clean:
	$(MAKE) -C $(DRIVERDIR) clean
