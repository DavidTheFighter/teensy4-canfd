# teensy4-canfd
A library written for the Teensy 4.x (i.MX RT 1062 MCU) to interface with the CANFD interface. Specifically, this library uses `imxrt-ral` and `teensy4-rs` to create a fully functioning interface. The code interface is a little specific to my own projects, but I'm planning on making it a little neater and better for more general use cases. Regardless it can act as a great place to start off another spin on a CAN implementation. Currently it only supports CANFD, no CAN2.0b.

For examples, look in the `/examples/` directory.
