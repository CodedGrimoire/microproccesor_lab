        AREA |.data|, DATA, READWRITE
data    DCB     0x01, 0xFF, 0x02, 0xC8   ; 4 bytes: 1, 255, 2, 200

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        LDR     R4, =data       ; Load base address of data into R4
        MOV     R0, #0          ; R0 will hold the running sum
        MOV     R5, #0          ; R5 will hold the carry flag (set manually)

        ; Load each byte and call Add_byte
        LDRB    R1, [R4]        ; Load byte 0 (1)
        BL      Add_byte

        LDRB    R1, [R4, #1]    ; Load byte 1 (255)
        BL      Add_byte

        LDRB    R1, [R4, #2]    ; Load byte 2 (2)
        BL      Add_byte

        LDRB    R1, [R4, #3]    ; Load byte 3 (200)
        BL      Add_byte

        BX      LR              ; Return

; ------------------------------
; Add_byte Function
; Input: R0 = current sum
;        R1 = byte to add
; Output: R0 = new sum
;         R5 = carry (0 or 1)
; ------------------------------
Add_byte
        ADDS    R0, R0, R1      ; Add with update to flags
        MOVCS   R5, #1          ; If carry set, set R5 = 1
        MOVCC   R5, #0          ; Else, R5 = 0
        BX      LR

        END
