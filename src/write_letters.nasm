.org 0x400

jmp i basic


write_H:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11111_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4

ret



; E
write_E:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b11111_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11111_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11111_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4
ret


; L
write_L:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11111_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4
ret

; O
write_O:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b11111_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11111_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4
ret


; W
write_W:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10101_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10101_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10101_000
mov mr r0, r2
call i go_down

mov ri r2, 0b01010_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4
ret

; SPACE
write_SPACE:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b00000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b00000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b00000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b00000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b00000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b00000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b00000_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4
ret

; R
write_R:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b11111_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11111_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11000_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10100_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10010_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4
ret


; D
write_D:

mov rr r3, r0
mov rr r4, r1

mov ri r2, 0b11110_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b10001_000
mov mr r0, r2
call i go_down

mov ri r2, 0b11110_000
mov mr r0, r2

mov rr r0, r3
mov rr r1, r4
ret



go_right:
add ri r1, 1
jc i inc_r0
ret

go_down:
add ri r1, 16
jc i inc_r0
ret


inc_r0:
add ri r0, 1
ret


basic:
mov ri r0, 0x08
mov ri r1, 0x00
call i write_H
call i go_right
call i write_E
call i go_right
call i write_L
call i go_right
call i write_L
call i go_right
call i write_O

call i go_right
call i write_SPACE

call i go_right
call i write_W
call i go_right
call i write_O
call i go_right
call i write_R
call i go_right
call i write_L
call i go_right
call i write_D

hlt
