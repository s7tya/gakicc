
#!/bin/sh
alias gakicc='RUSTFLAGS=-Awarnings cargo run -q --'

tmp=`mktemp -d /tmp/gakicc-test-XXXXXX`
trap 'rm -rf $tmp' INT TERM HUP EXIT
echo > $tmp/empty.c

check() {
    if [ $? -eq 0 ]; then
        echo "testing $1 ... passed"
    else
        echo "testing $1 ... failed"
        exit 1
    fi
}

# -o
rm -f $tmp/out
gakicc -o $tmp/out $tmp/empty.c
[ -f $tmp/out ]
check -o

# --help
gakicc --help 2>&1 | grep -q gakicc
check --help

echo OK