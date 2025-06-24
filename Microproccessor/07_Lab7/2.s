        AREA |.rodata|, DATA, READONLY
Matrix1 DCD     1, 2, 3, 4          ; 2x2 matrix
Matrix2 DCD     5, 6, 7, 8          ; 2x2 matrix

        AREA |.data|, DATA, READWRITE
Result  DCD     0, 0, 0, 0          ; 2x2 result matrix (initialized to 0)

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =Matrix1      ; Base address of Matrix1
        LDR     R1, =Matrix2      ; Base address of Matrix2
        LDR     R2, =Result       ; Base address of Result matrix

        ; Multiply first row of Matrix1 by first column of Matrix2
        LDR     R3, [R0]          ; Load Matrix1[0][0] (1)
        LDR     R4, [R1]          ; Load Matrix2[0][0] (5)
        MUL     R5, R3, R4        ; Multiply
        LDR     R6, [R0, #4]      ; Load Matrix1[0][1] (2)
        LDR     R7, [R1, #8]      ; Load Matrix2[1][0] (7)
        MUL     R8, R6, R7        ; Multiply
        ADD     R9, R5, R8        ; Add results
        STR     R9, [R2]          ; Store in Result[0][0]

        ; Multiply first row of Matrix1 by second column of Matrix2
        LDR     R3, [R0]          ; Load Matrix1[0][0] (1)
        LDR     R4, [R1, #4]      ; Load Matrix2[0][1] (6)
        MUL     R5, R3, R4        ; Multiply
        LDR     R6, [R0, #4]      ; Load Matrix1[0][1] (2)
        LDR     R7, [R1, #12]     ; Load Matrix2[1][1] (8)
        MUL     R8, R6, R7        ; Multiply
        ADD     R9, R5, R8        ; Add results
        STR     R9, [R2, #4]      ; Store in Result[0][1]

        ; Multiply second row of Matrix1 by first column of Matrix2
        LDR     R3, [R0, #8]      ; Load Matrix1[1][0] (3)
        LDR     R4, [R1]          ; Load Matrix2[0][0] (5)
        MUL     R5, R3, R4        ; Multiply
        LDR     R6, [R0, #12]     ; Load Matrix1[1][1] (4)
        LDR     R7, [R1, #8]      ; Load Matrix2[1][0] (7)
        MUL     R8, R6, R7        ; Multiply
        ADD     R9, R5, R8        ; Add results
        STR     R9, [R2, #8]      ; Store in Result[1][0]

        ; Multiply second row of Matrix1 by second column of Matrix2
        LDR     R3, [R0, #8]      ; Load Matrix1[1][0] (3)
        LDR     R4, [R1, #4]      ; Load Matrix2[0][1] (6)
        MUL     R5, R3, R4        ; Multiply
        LDR     R6, [R0, #12]     ; Load Matrix1[1][1] (4)
        LDR     R7, [R1, #12]     ; Load Matrix2[1][1] (8)
        MUL     R8, R6, R7        ; Multiply
        ADD     R9, R5, R8        ; Add results
        STR     R9, [R2, #12]     ; Store in Result[1][1]

stop    B stop

        END
