        AREA |.data|, DATA, READWRITE
Data    DCD     1, 255, 2, 200 ; Example byte data

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R5, =Data           ; Load address of Data into R5
        LDR     R0, [R5]            ; Load first byte (1)
        LDR     R1, [R5, #4]        ; Load second byte (255)
        LDR     R2, [R5, #8]        ; Load third byte (2)
        LDR     R3, [R5, #12]       ; Load fourth byte (200)
        
        ADD     R4, R0, R1          ; Add first two bytes
        ADC     R4, R4, R2          ; Add third byte with carry
        ADC     R4, R4, R3          ; Add fourth byte with carry
        
        MOV     R5, R4              ; Store result with carry in R5
        BX      LR

        END
