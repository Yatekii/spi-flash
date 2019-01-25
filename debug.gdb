# This connects to the GDB server running locally.
#   - for openocd, use port :3333
#   - for JLinkGDBServer, use port :2331
target remote :2331

# Due to https://github.com/rust-embedded/cortex-m-rt/issues/139,
#   we will get an infinite backtrace on a panic!(). Set a finite
#   limit to the backtrace to prevent the debugger falling into
#   an endless loop trying to read the backtrace
set backtrace limit 32

# Load the specified firmware onto the device
load

# Reset the target device before running (using JLinkGDBServer)
mon reset 0

# Reset the target device before running (using openocd)
# reset