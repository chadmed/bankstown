# SPDX-Licence-Identifier: MIT
# Copyright (C) 2023 James Calligeros <jcalligeros99@gmail.com>

LIBDIR ?= /usr/lib64

default:
	cargo build --release

install:
	install -dD $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/
	install -m0655 target/release/libbankstown.so $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/bankstown.so
	install -m0644 bankstown.ttl $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/bankstown.ttl
	install -m0644 manifest.ttl $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/manifest.ttl

uninstall:
	rm -rf $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/
