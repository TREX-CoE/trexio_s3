PREFIX ?= /usr/local
INCLUDEDIR = $(PREFIX)/include
LIBDIR = $(PREFIX)/lib
PKGCONFIGDIR = $(LIBDIR)/pkgconfig

HEADER = include/trexio_s3_rust.h
TARGET = target/release

all: $(TARGET)/libtrexio_s3.so include/trexio_s3_rust.h pkg-config

$(TARGET)/libtrexio_s3.so:
	cargo build --release

check:
	cargo test

pkg-config:
	@sed "s|@prefix@|$(PREFIX)|" trexio_s3.pc.in > trexio_s3.pc

install: all 
	# Install the C header
	install -d $(INCLUDEDIR)
	install -m 644 $(HEADER) $(INCLUDEDIR)

	# Install pkg-config files
	install -d $(PKGCONFIGDIR)
	install -m 644 trexio_s3.pc $(PKGCONFIGDIR)

	# Install the compiled Rust library (both .so and .a if desired)
	install -d $(LIBDIR)
	install -m 644 $(TARGET)/libtrexio_s3.a $(LIBDIR)
	install -m 755 $(TARGET)/libtrexio_s3.so $(LIBDIR)

uninstall:
	rm -f $(INCLUDEDIR)/trexio_s3_rust.h
	rm -f $(LIBDIR)/libtrexio_s3.a
	rm -f $(LIBDIR)/libtrexio_s3.so
	rm -f $(PKGCONFIGDIR)/trexio_s3.pc

.PHONY: all clean pkg-config
