        AREA |.rodata|, DATA, READONLY

; -------- 32-bit input variables --------
X     DCD     0x0000000A      ; X = 10
Y     DCD     0x00000005      ; Y = 5
Z     DCD     0x00000006      ; Z = 6
P     DCD     0x00000007      ; P = 7
Q     DCD     0x00000008      ; Q = 8

        AREA |.data|, DATA, READWRITE

Largest      DCD     0               ; Store the largest value here

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        ; Load the numbers into registers
        LDR     R0, =X
        LDR     R1, [R0]        ; R1 = X

        LDR     R0, =Y
        LDR     R2, [R0]        ; R2 = Y

        LDR     R0, =Z
        LDR     R3, [R0]        ; R3 = Z

        LDR     R0, =P
        LDR     R4, [R0]        ; R4 = P

        LDR     R0, =Q
        LDR     R5, [R0]        ; R5 = Q

        ; Start assuming R1 is the largest
        MOV     R6, R1          ; R6 = max

        ; Compare R6 with R2 (Y)
        CMP     R2, R6
        BLS     skip2 ;Branch to skip2 if R6 <= R2
        MOV     R6, R2
skip2
        ; Compare R6 with R3 (Z)
        CMP     R3, R6
        BLS     skip3
        MOV     R6, R3
skip3
        ; Compare R6 with R4 (P)
        CMP     R4, R6
        BLS     skip4
        MOV     R6, R4
skip4
        ; Compare R6 with R5 (Q)
        CMP     R5, R6
        BLS     skip5
        MOV     R6, R5
skip5
        ; Store the result
        LDR     R0, =Largest
        STR     R6, [R0]

        ; Infinite loop
stop    B stop

        END
