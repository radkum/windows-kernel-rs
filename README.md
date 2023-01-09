# windows-kernel-rs

sysmon-driver-rust - rust driver based on https://github.com/zodiacon/windowskernelprogrammingbook/tree/master/chapter09/SysMon 

Some ideas taken from:  [Writing a kernel driver with Rust.](https://not-matthias.github.io/kernel-driver-with-rust/)

todo: 
- test if KeGetCurrentIrql works properly
- you mixed passing args by reference and pointer. What can you do about it?
- 
- check an allocation, maybe don't throw panic but Error?
- find and use KiIrqlLevel function in minifilter
- check a book windowskernelprogramming2e and upgrade driver and minifilter
- create a Firewall, Antiransomware or kernel hook engine in Rust