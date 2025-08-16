riscv64-linux-gnu-gcc -E -P -C test/2kmcc/2kmcc.c -std=c11 -fno-common -o - | RUSTFLAGS=-Awarnings cargo run -q -- -o test/2kmcc/2kmcc.s -
riscv64-linux-gnu-gcc test/2kmcc/2kmcc.s -static -o test/2kmcc/2kmcc.exe
cat test/2kmcc/2kmcc.c | xargs -0 -I XX qemu-riscv64 ./test/2kmcc/2kmcc.exe XX > test/2kmcc/2kmcc-2.s
gcc test/2kmcc/2kmcc-2.s -static -o test/2kmcc/2kmcc-2.exe
cat test/2kmcc/2kmcc.c | xargs -0 -I XX test/2kmcc/2kmcc-2.exe XX > test/2kmcc/2kmcc-3.s

echo "Comparing 2kmcc-2.s and 2kmcc-3.s:"
diff -u test/2kmcc/2kmcc-2.s test/2kmcc/2kmcc-3.s  && echo -e "OK\n"