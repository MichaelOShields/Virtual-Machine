.org 0x400


mov ri r0, 8
mov ri r1, 0

mov ri r2, 255

core_loop:

mov mr r0, r2

add ri r1, 1

jc i inc_r0

jmp i core_loop


inc_r0:
add ri r0, 1

cmp ri r0, 17
jz i halt

jmp i core_loop



halt:
hlt