        AREA |.rodata|, DATA, READONLY

BATTERY_LEVEL   DCD     0x00000014      ; Example: 20
LOAD_STATUS     DCD     0x00000001      ; Example: 1 = Heavy

        AREA |.data|, DATA, READWRITE

MODE            DCD     0               ; Output Mode

        AREA |.text|, CODE, READONLY
        ENTRY
        EXPORT main

main
        ; Load BATTERY_LEVEL into R1
        LDR     R0, =BATTERY_LEVEL
        LDR     R1, [R0]

        ; Load LOAD_STATUS into R2
        LDR     R0, =LOAD_STATUS
        LDR     R2, [R0]

        ; ----------- Condition 1: Battery < 20 AND Load == 1 → MODE = 1 (Low-power)
        MOV     R3, #20
        CMP     R1, R3
        BGE     check_high_perf          ; Skip if battery >= 20

        CMP     R2, #1
        BNE     check_high_perf          ; Skip if load != 1

        ; Set MODE = 1
        MOV     R4, #1
        LDR     R0, =MODE
        STR     R4, [R0]
        B       done

check_high_perf
        ; ----------- Condition 2: Battery > 80 → MODE = 3 (High-performance)
        MOV     R3, #80
        CMP     R1, R3
        BLE     set_normal               ; Skip if battery <= 80

        ; Set MODE = 3
        MOV     R4, #3
        LDR     R0, =MODE
        STR     R4, [R0]
        B       done

set_normal
        ; ----------- Else → MODE = 2 (Normal)
        MOV     R4, #2
        LDR     R0, =MODE
        STR     R4, [R0]

done
        ; Infinite loop
        B       done

        END
