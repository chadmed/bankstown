# SPDX-Licence-Identifier: MIT
# Copyright (C) 2023 James Calligeros <jcalligeros99@gmail.com>

LIBDIR ?= /usr/lib64

default:
	cargo build --release

install:
	install -dDm0755 $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/
	install -pm0755 target/release/libbankstown.so $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/bankstown.so
	install -pm0644 bankstown.ttl $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/bankstown.ttl
	install -pm0644 manifest.ttl $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/manifest.ttl

uninstall:
	rm -rf $(DESTDIR)/$(LIBDIR)/lv2/bankstown.lv2/
