        AREA |.rodata|, DATA, READONLY

; -------- 32-bit input variables --------
X     DCD     0x0000000A      ; X = 10
Y     DCD     0x00000005      ; Y = 5
Z     DCD     0x00000006      ; Z = 6
P     DCD     0x00000007      ; P = 7
Q     DCD     0x00000008      ; Q = 8
N     DCD     0x00000005      ; N = 5

        AREA |.data|, DATA, READWRITE

; -------- Output variable --------
SUM      DCD     0               ; Store the average value here

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        ; -------- Load each 32-bit number from memory into registers --------
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
		
		

        ; -------- Add all numbers together --------
        ADD     R6, R1, R2      ; R6 = X + Y
        ADD     R6, R6, R3      ; R6 = R6 + Z
        ADD     R6, R6, R4      ; R6 = R6 + P
        ADD     R6, R6, R5      ; R6 = R6 + Q

        

        ; -------- Store the result in memory --------
        LDR     R0, =SUM
        STR     R6, [R0]        ; Store R6 to SUM

        ; -------- Infinite loop to end program --------
stop    B stop

        END
