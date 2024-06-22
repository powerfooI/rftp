# rFTP - FTP server written in Rust

## Introduction

`rFTP` is a simple FTP server written in Rust. It is designed to be a learning project for me to understand the basics and asynchronous I/O in Rust. The project is built on top of the [Tokio](https://tokio.rs/) runtime, which provides asynchronous I/O and networking support.

## Features

### Basic Commands

- `USER/PASS`
- `PORT/PASV`
- `RETR/STOR`
- `ABOR/QUIT`
- `SYST/TYPE/STAT`
- `RNFR/RNTO`
- `PWD/CWD/MKD/RMD/CDUP`
- `LIST/NLST`
- `DELE`
- `NOOP`

### Advanced Commands

- `REST`
- `STOU`
- `APPE`
- `ALLO`
- `FEAT`
- `MDTM`
- [ ] `SITE`

## References

- [rfc959](https://www.ietf.org/rfc/rfc959.txt)
- [epoll](https://man7.org/linux/man-pages/man7/epoll.7.html)
- [kqueue](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kqueue.2.html)
- [Kernel Queues: An Alternative to File System Events](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kqueue.2.html)
- [The Rust Programming Language](https://doc.rust-lang.org/book/title-page.html)
- [Tokio - An Asynchronous Rust Runtime](https://tokio.rs/)
