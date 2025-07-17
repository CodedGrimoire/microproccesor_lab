        AREA |.data|, DATA, READWRITE
BCDData  DCD     0x23 ; Example BCD (2 and 3)

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R0, =BCDData        ; Load address of BCD data into R0
        LDR     R1, [R0]            ; Load BCD data (0x23)
        BL      BCD_binary          ; Call BCD to binary conversion function
        BX      LR                  ; Return from main

BCD_binary
        ; Convert BCD to Binary
        AND     R0, R1, #0x0F       ; Extract ones place (R0 = 3)
        MOV     R2, R0              ; Store ones place in R2
        LSR     R1, R1, #4          ; Logical shift right to get tens place
        AND     R1, R1, #0x0F       ; Mask to get tens place (R1 = 2)
        MOV     R3, R1              ; Store tens place in R3
        ADD     R0, R2, R3, LSL #4  ; Convert BCD to binary (R0 = 2 * 10 + 3 = 23)
        MOV     PC, LR              ; Return from BCD_binary function

        END
 
