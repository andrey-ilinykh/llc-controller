# Connect to OpenOCD running on localhost port 3333
target extended-remote :3333

# Print demangled symbols
set print asm-demangle on

# Set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# Detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# Load will flash the code
load

# Enable semihosting
monitor arm semihosting enable

# Start the process but immediately halt the processor
stepi
