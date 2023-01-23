# windows-kernel-rs

sysmon-driver-rust - rust driver based on https://github.com/zodiacon/windowskernelprogrammingbook/tree/master/chapter09/SysMon 


Some ideas taken from:  [Writing a kernel driver with Rust.](https://not-matthias.github.io/kernel-driver-with-rust/)

todo:
- upgrade delprotect to protect other processes than cmd
- check a book windowskernelprogramming2e and upgrade driver and minifilter
- create a Firewall, Antiransomware or kernel hook engine in Rust

onhold
- you mixed passing args by reference and pointer. What can you do about it?
