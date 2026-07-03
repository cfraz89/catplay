obj-m := g_iphone.o iap2_char.o iap2_scan.o
#iap2_scan-y := iap2_scan.o
#iap2_char-y := iap2_char.o

g_iphone-y := iphone_mod.o iphone_dev.o iphone_class.o f_iphone.o g_iphone_impl.o composite.o 

KDIR := /lib/modules/$(shell uname -r)/build

PWD := $(shell pwd)

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

insmod:
	insmod iap2_char.ko
	insmod iap2_scan.ko
	modprobe libcomposite
	insmod g_iphone.ko
