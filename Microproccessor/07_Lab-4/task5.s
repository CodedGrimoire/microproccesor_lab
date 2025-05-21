        AREA |.data|, DATA, READWRITE
decimal_num  DCD     25              ; Decimal number: 25
binary_num   DCD     0               ; To store the binary value

        AREA DecimalToBinary, CODE, READONLY
        ENTRY
        EXPORT main

main
        ; Load decimal number into r0
        LDR     r0, =decimal_num
        LDR     r1, [r0]        ; r1 = decimal_num (25)

        ; Prepare to convert to binary
        LDR     r2, =binary_num ; Address of binary_num
        MOV     r3, #0          ; Clear r3 (to store binary result)

convert_loop
        CMP     r1, #0          ; Check if decimal_num is 0
        BEQ     done            ; If it is 0, we're done

        ; Get remainder (binary bit) using bitwise AND with 1
        AND     r4, r1, #1      ; r4 = r1 & 1 (Get LSB of decimal_num)

        ; Shift r1 right by 1 (divide by 2) for next iteration
        LSR     r1, r1, #1      ; Logical Shift Right

        ; Shift binary result to the left and insert bit into the result
        LSL     r3, r3, #1      ; Shift left result
        ORR     r3, r3, r4      ; Set the LSB of r3 with the current bit

        B       convert_loop    ; Repeat until r1 becomes 0

done
        ; Store the result in binary_num
        STR     r3, [r2]        ; Store the binary result

        MOV     r0, #0          ; Return 0 (end of program)
        BX      LR              ; Return from main

        END
