https://github.com/riscv-software-src/riscv-pk/issues/269

## Build

### Build with musl
```bash
docker run -it --rm -v `pwd`:/srv --platform linux/riscv64 ubuntu:noble bash
```
in the container
```bash
apt-get install -y --no-install-recommends musl musl-dev musl-tools
cd /srv
musl-gcc -static -march=rv64gc src/main.c -o hello
```
```bash
# file ./hello
./hello: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, with debug_info, not stripped
# ls -lh ./hello
-rwxr-xr-x 1 root root 19K Jun 11 13:14 ./hello
# readelf -hl ./hello
ELF Header:
  Magic:   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00 
  Class:                             ELF64
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0
  Type:                              EXEC (Executable file)
  Machine:                           RISC-V
  Version:                           0x1
  Entry point address:               0x10178
  Start of program headers:          64 (bytes into file)
  Start of section headers:          17224 (bytes into file)
  Flags:                             0x5, RVC, double-float ABI
  Size of this header:               64 (bytes)
  Size of program headers:           56 (bytes)
  Number of program headers:         5
  Size of section headers:           64 (bytes)
  Number of section headers:         25
  Section header string table index: 24

Program Headers:
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  RISCV_ATTRIBUT 0x0000000000002150 0x0000000000000000 0x0000000000000000
                 0x0000000000000053 0x0000000000000000  R      0x1
  LOAD           0x0000000000000000 0x0000000000010000 0x0000000000010000
                 0x0000000000001418 0x0000000000001418  R E    0x1000
  LOAD           0x0000000000001f48 0x0000000000012f48 0x0000000000012f48
                 0x00000000000001b8 0x0000000000000800  RW     0x1000
  GNU_STACK      0x0000000000000000 0x0000000000000000 0x0000000000000000
                 0x0000000000000000 0x0000000000000000  RW     0x10
  GNU_RELRO      0x0000000000001f48 0x0000000000012f48 0x0000000000012f48
                 0x00000000000000b8 0x00000000000000b8  R      0x1

 Section to Segment mapping:
  Segment Sections...
   00     .riscv.attributes 
   01     .text .rodata .eh_frame 
   02     .init_array .fini_array .data.rel.ro .got .got.plt .data .bss 
   03     
   04     .init_array .fini_array .data.rel.ro .got .got.plt 
```

execute hello will output message:
```bash
./hello 
hello riscv!
```

### Build with gcc
```bash
apt-get install -y --no-install-recommends build-essential file
cd /srv
gcc -g -static -march=rv64gc src/main.c -o hello
```

```bash
# file ./hello
./hello: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, BuildID[sha1]=d8c90155462a06501804a7fe6f07d358fd04eaf7, for GNU/Linux 4.15.0, with debug_info, not stripped
# ls -hl ./hello
-rwxr-xr-x 1 root root 550K Jun 11 14:12 ./hello
# readelf -hl ./hello
ELF Header:
  Magic:   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00 
  Class:                             ELF64
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0
  Type:                              EXEC (Executable file)
  Machine:                           RISC-V
  Version:                           0x1
  Entry point address:               0x10494
  Start of program headers:          64 (bytes into file)
  Start of section headers:          560480 (bytes into file)
  Flags:                             0x5, RVC, double-float ABI
  Size of this header:               64 (bytes)
  Size of program headers:           56 (bytes)
  Number of program headers:         7
  Size of section headers:           64 (bytes)
  Number of section headers:         32
  Section header string table index: 31

Program Headers:
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  RISCV_ATTRIBUT 0x00000000000738cb 0x0000000000000000 0x0000000000000000
                 0x0000000000000057 0x0000000000000000  R      0x1
  LOAD           0x0000000000000000 0x0000000000010000 0x0000000000010000
                 0x000000000006df64 0x000000000006df64  R E    0x1000
  LOAD           0x000000000006e378 0x000000000007e378 0x000000000007e378
                 0x0000000000005528 0x000000000000a938  RW     0x1000
  NOTE           0x00000000000001c8 0x00000000000101c8 0x00000000000101c8
                 0x0000000000000044 0x0000000000000044  R      0x4
  TLS            0x000000000006e378 0x000000000007e378 0x000000000007e378
                 0x0000000000000018 0x0000000000000058  R      0x8
  GNU_STACK      0x0000000000000000 0x0000000000000000 0x0000000000000000
                 0x0000000000000000 0x0000000000000000  RW     0x10
  GNU_RELRO      0x000000000006e378 0x000000000007e378 0x000000000007e378
                 0x0000000000003c88 0x0000000000003c88  R      0x1

 Section to Segment mapping:
  Segment Sections...
   00     .riscv.attributes 
   01     .note.gnu.build-id .note.ABI-tag .rela.dyn .text .rodata .stapsdt.base .eh_frame .gcc_except_table 
   02     .tdata .preinit_array .init_array .fini_array .data.rel.ro .got .got.plt .data .sdata .bss 
   03     .note.gnu.build-id .note.ABI-tag 
   04     .tdata .tbss 
   05     
   06     .tdata .preinit_array .init_array .fini_array .data.rel.ro .got .got.plt 
```

### Build with riscv64-unknown-elf-gcc
```bash
apt-get install -y --no-install-recommends gcc-riscv64-unknown-elf
cd /srv
riscv64-unknown-elf-gcc -nostdlib -nostartfiles -march=rv64gc -T src/linker.ld src/start.S src/hello.c -o hello
```

```bash
# file ./hello
./hello: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, not stripped
# ls -hl ./hello
-rwxr-xr-x 1 root root 5.1K Jun 11 14:13 ./hello
# readelf -hl ./hello
ELF Header:
  Magic:   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00 
  Class:                             ELF64
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0
  Type:                              EXEC (Executable file)
  Machine:                           RISC-V
  Version:                           0x1
  Entry point address:               0x80000000
  Start of program headers:          64 (bytes into file)
  Start of section headers:          4680 (bytes into file)
  Flags:                             0x5, RVC, double-float ABI
  Size of this header:               64 (bytes)
  Size of program headers:           56 (bytes)
  Number of program headers:         2
  Size of section headers:           64 (bytes)
  Number of section headers:         7
  Section header string table index: 6

Program Headers:
  Type           Offset             VirtAddr           PhysAddr
                 FileSiz            MemSiz              Flags  Align
  RISCV_ATTRIBUT 0x0000000000001022 0x0000000000000000 0x0000000000000000
                 0x0000000000000053 0x0000000000000000  R      0x1
  LOAD           0x0000000000001000 0x0000000080000000 0x0000000080000000
                 0x0000000000000022 0x0000000000000022  R E    0x1000

 Section to Segment mapping:
  Segment Sections...
   00     .riscv.attributes 
   01     .text 
```
## Run with qemu
```bash
qemu-riscv64-static ./hello
```
| Compiler | Status |
|----------|--------|
| musl-gcc | ✅ Works |
| gcc | ✅ Works |
| riscv64-unknown-elf-gcc | ❌ Fails |


## Run with spike
```bash
spike --isa=rv64gc /opt/riscv/riscv64-unknown-elf/bin/pk hello
```

| Compiler | spike pk | spike pk -s |
|----------|----------|-------------|
| musl-gcc | ✅ Works | ❌ Fails |
| gcc | ❌ Fails | ❌ Fails |
| riscv64-unknown-elf-gcc | ❌ Fails | ❌ Fails |

## Toubleshooting

### stuck if hello is built with gcc
```bash
apt-get install -y --no-install-recommends build-essential file
cd /srv
gcc -g -static -march=rv64gc src/main.c -o hello
```

```bash
# file ./hello
./hello: ELF 64-bit LSB executable, UCB RISC-V, RVC, double-float ABI, version 1 (SYSV), statically linked, BuildID[sha1]=4fad791ffe1d029c391d3d56be03cb0a9a539ceb, for GNU/Linux 4.15.0, with debug_info, not stripped
# ls -lh ./hello
-rwxr-xr-x 1 root root 550K Jun 11 13:17 ./hello
```
if hello is built by the command `gcc -g -static -march=rv64gc src/main.c -o hello`, it will stuck when executing `spike pk hello`.

```bash
core   0: exception trap_illegal_instruction, epc 0xffffffc000002004
core   0:           tval 0x00000000c00027f3
```

### stuck if pk with '-s' option
```bash
spike  --isa=rv64gc /opt/riscv/riscv64-unknown-elf/bin/pk -s hello
```
Ctrl + c and debug
```bash
(spike) insn 0
0x0000000034011173 csrrw   sp, mscratch, sp
(spike) r 1
core   0: >>>>  trap_vector
core   0: 0x0000000080003c74 (0x34011173) csrrw   sp, mscratch, sp
(spike) r 1
core   0: 0x0000000080003c78 (0x1a010863) beqz    sp, pc + 432
(spike) r 1
core   0: 0x0000000080003e28 (0x00000e18) c.addi4spn a4, sp, 784
(spike) r 1
core   0: 0x0000000080003e2a (0x00000061) c.addi  zero, 24
(spike) r 1
core   0: 0x0000000080003e2c (0x0000ffc0) c.sd    s0, 184(a5)
core   0: exception trap_store_address_misaligned, epc 0x0000000080003e2c
core   0:           tval 0x00000000000000b9
```

### stuck if hello is built with riscv64-unknown-elf-gcc
```bash
apt-get install -y --no-install-recommends gcc-riscv64-unknown-elf
cd /srv
riscv64-unknown-elf-gcc -nostdlib -nostartfiles -march=rv64gc -T src/linker.ld src/start.S src/hello.c -o hello
```
Ctrl + c and debug
```
core   0: 0x0000000080003da0 (0x000300e7) jalr    t1
core   0: exception trap_instruction_access_fault, epc 0xffffffc000610e18
core   0:           tval 0xffffffc000610e18
```
