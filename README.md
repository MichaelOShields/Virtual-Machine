A simple 8 bit-registered (16 bit-addressed) system virtual machine I made in Rust.
Although I've always known that all code I write is "compiled into human-unreadable machine code," I was curious as to what the process looked like in person, so I created this.
It features a simulated CPU with a custom ISA, 64-KB memory, program protection levels, etc.
I also wrote a two-pass assembler for the ISA which greatly simplified program writing.
I wrote the kernel in the assembler. It features a round-robin scheduler, context switching, syscall handler, CPU exit trap handler, and more.
