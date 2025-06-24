        AREA |.data|, DATA, READWRITE
Counter  DCD     0                ; Start at 00

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        MOV     R0, #0              ; Start counter at 0
Loop
        ADD     R0, R0, #1          ; Increment counter
        MOV     R1, R0              ; Move to R1 for BCD display
        BL      Delay               ; Call delay function
        CMP     R0, #100            ; Check if 99 reached
        BNE     Loop                ; Loop if not
        BX      LR

Delay
        ; 1-second delay calculation here
        ; Insert appropriate delay logic based on system clock
        BX      LR
