SHELL := /bin/sh

CC := riscv64-linux-gnu-gcc
CFLAGS := -std=c11 -g -fno-common

GAKICC := RUSTFLAGS=-Awarnings cargo run -q --
SH := qemu-riscv64

TEST_SRCS := $(wildcard test/*.c)
TESTS := $(TEST_SRCS:.c=.exe)

.PHONY: test clean

test/%.s: test/%.c
	$(CC) -o- -E -P -C $< | $(GAKICC) -o $@ -

test/%.exe: test/%.s
	$(CC) -static -o $@ $< -xc test/common

test: clean $(TESTS)
	for i in $(TESTS); do echo $$i; $(SH) ./$$i || exit 1; echo; done
	$(GAKICC) -o test/donut/donut.s test/donut/donut.c && $(CC) -static -o test/donut/donut.exe test/donut/donut.s && $(SH) ./test/donut/donut.exe | diff test/donut/snap.txt - 2>&1 > /dev/null && echo "OK"
	test/driver.sh

clean:
	rm -f test/*.s test/*.exe test/donut/donut.s test/donut/donut.exe
