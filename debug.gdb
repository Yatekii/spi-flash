target remote :2331
#monitor reset halt
load
mon reset 0
#monitor semihosting enable
#continue