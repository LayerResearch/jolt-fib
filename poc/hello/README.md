https://github.com/riscv-software-src/riscv-pk/issues/269

## Build

Install qemu on host
```bash
apt-get update && apt-get install -y --no-install-recommends qemu-user-static
```

Create a container for compilation.
```bash
docker run -it --rm -v `pwd`:/srv --platform linux/riscv64 ubuntu:noble bash
```

### Build with musl

#### In Container
```bash
apt-get update && apt-get install -y --no-install-recommends build-essential musl musl-dev musl-tools
cd /srv
musl-gcc -static -march=rv64gc src/main.c -o hello
```
```bash
# file ./hello
./hello: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, with debug_info, not stripped
# ls -lh ./hello
-rwxr-xr-x 1 root root 19K Jun 11 13:14 ./hello
# ./hello 
hello riscv!
```

#### On host
```bash
$ qemu-riscv64-static ./hello
hello riscv!
$ spike --isa=rv64gc /opt/riscv/riscv64-unknown-elf/bin/pk ./hello
hello riscv!
```

### Build with gcc

### In Container
```bash
apt-get update && apt-get install -y --no-install-recommends build-essential file
cd /srv
gcc -g -static -march=rv64gc src/main.c -o hello
```

```bash
# file ./hello
./hello: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, BuildID[sha1]=d8c90155462a06501804a7fe6f07d358fd04eaf7, for GNU/Linux 4.15.0, with debug_info, not stripped
# ls -hl ./hello
-rwxr-xr-x 1 root root 550K Jun 11 14:12 ./hello
# ./hello 
hello riscv!
```

#### On host

```sh
$ qemu-riscv64-static ./hello
hello riscv!
$ spike  --isa=rv64gc /opt/riscv/riscv64-unknown-elf/bin/pk ./hello
(spike) r 1
core   0: >>>>  trap_vector
core   0: 0x0000000080003dc8 (0x00003ef8) c.fld   fa4, 248(a3)
core   0: exception trap_load_access_fault, epc 0x0000000080003dc8
core   0:           tval 0x00000000000000f8
```

### Build with riscv64-unknown-elf-gcc
#### In Container
```bash
apt-get update && apt-get install -y --no-install-recommends gcc-riscv64-unknown-elf picolibc-riscv64-unknown-elf
cd /srv
riscv64-unknown-elf-gcc -nostdlib -nostartfiles -march=rv64gc -T src/linker.ld src/start.S src/number.c -o number
```

```bash
# file ./number
./number: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, not stripped
# ls -hl ./number
-rwxr-xr-x 1 root root 5.1K Jun 11 14:13 ./number
# ./number 
# echo $?
42
```

### On host
```bash
$ qemu-riscv64-static ./number
$ echo $?
42
$ spike  --isa=rv64gc /opt/riscv/riscv64-unknown-elf/bin/pk ./number
$ echo $?
42
```

## Results
```bash
qemu-riscv64-static ./hello
```
| Compiler | qemu-riscv64-static | spike + pk |
|----------|---------------------|--------|
| musl-gcc | ✅ Works | ✅ Works |
| gcc | ✅ Works | ❌ Fails |
| riscv64-unknown-elf-gcc | ✅ Works | ✅ Works |

