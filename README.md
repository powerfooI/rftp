# RFTP - FTP server written in Rust

## Developing Plan

### Basic Commands

* `USER/PASS`
* `PORT/PASV`
* `RETR/STOR`
* `ABOR/QUIT`
* `SYST/TYPE`
* `RNFR/RNTO`
* `LIST`

### Advanced Commands

* `REST`
* `PWD/CWD/MKD/RMD`
* `DELE`
* `STAT`
* `STOU`
* `APPE`
* `ALLO`


## References 

* [RFC959](https://www.ietf.org/rfc/rfc959.txt)
* [epoll](https://man7.org/linux/man-pages/man7/epoll.7.html)