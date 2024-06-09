#!/usr/bin/env python3
from ftplib import FTP

class Client(object):
    def __init__(self, addr='127.0.0.1', port=2121, username='admin', password='123456'):
        self.cli = FTP()
        self.addr = addr
        self.port = port
        self.cli.connect(host=addr, port=port)
        self.cli.login(user=username,passwd=password)
        self.is_pasv = True
        self.is_debug = True
        self.cli.set_pasv(self.is_pasv)
        self.cmd_set = ['cd', 'ls', 'mv', 'upload', 'download','syst',
                         'pwd', 'rmdir', 'mkdir', 'quit', 'set-pasv', 'type']
        self.cmd = b''
        self.param = b''
        self.buffer = b''

    def handle_input(self):
        sp_pos = self.buffer.find(' ')
        if sp_pos > 0 :
            self.cmd = self.buffer[:sp_pos]
            self.param = self.buffer[sp_pos+1:]
        else:
            self.cmd = self.buffer
        self.buffer = b''
    
    def exec_cmd(self):
        if self.cmd not in self.cmd_set:
            print('==> Only support following cmd:\n'+','.join(self.cmd_set))
        else:
            try:
                if self.cmd == 'cd':
                    self.cli.sendcmd('CWD '+ self.param)
                    print('==> directory changed!')
                elif self.cmd == 'ls':
                    self.cli.dir(self.param, lambda line: print(line, '123'))
                elif self.cmd == 'mv':
                    two_param = self.param.split()
                    if len(two_param) < 2:
                        print('==> Too few arguments!')
                    else:
                        self.cli.sendcmd('RNFR ' + two_param[0])
                        self.cli.sendcmd('RNTO ' + two_param[1])
                        print('==> mv seccessfully!')
                elif self.cmd == 'upload':
                    res = self.cli.storbinary('STOR ' + self.param, open(self.param, 'rb'))
                    print(res)
                    print('==> upload successfully!')
                elif self.cmd == 'download':
                    res = self.cli.retrbinary('RETR ' + self.param, open(self.param, 'wb').write)
                    print(res)
                    print('==> download successfully!')
                elif self.cmd == 'pwd':
                    print('==> ' + self.cli.sendcmd('PWD').split()[1])
                elif self.cmd == 'rmdir':
                    self.cli.rmd(self.param)
                    print('==> rmdir {} successfully!'.format(self.param))
                elif self.cmd == 'mkdir':
                    self.cli.mkd(self.param)
                    print('==> create directory {} successfully!'.format(self.param))
                elif self.cmd == 'quit':
                    self.cli.quit()
                    return -1
                elif self.cmd == 'syst':
                    print(self.cli.sendcmd('SYST'))
                elif self.cmd == 'type':
                    print(self.cli.sendcmd('TYPE ' + self.param))
                elif self.cmd == 'set-pasv':
                    self.is_pasv = not self.is_pasv
                    self.cli.set_pasv(self.is_pasv)

            except Exception as e:
                print("[Error]", e)
        self.cmd = ''
        self.param = ''
        return 0
    
    def run(self):
        print("==> Client is running, connect to {} on {}".format(self.addr, self.port))
        while True:
            self.buffer = input('<== ')
            self.handle_input()
            if self.cmd == 'debug':
                self.is_debug = not self.is_debug
                self.cli.debugging = self.is_debug
                continue
            if not self.is_debug:
                if self.exec_cmd() < 0:
                    print("==> Client quit. Thank you for using.")
                    break
            else:
                try:
                    self.cli.sendcmd(' '.join([self.cmd, self.param]))
                except Exception as e:
                    print("[Error]", e)

if __name__ == '__main__':
    # addr = input('==> [Configure]input the addr(default is 127.0.0.1): ')
    # port = input('==> [Configure]input the port(default is 21): ')
    # username = input('==> [Configure]input the username(default is anonymous): ')
    # password = input('==> [Configure]input the password(default is uuu@): ')
    # if not addr:
    #     addr = '127.0.0.1'
    # if not port:
    #     port = 2121
    # if not username:
    #     # username = 'anonymous'
    #     username = 'admin'
    # if not password:
    #     # password = 'uuu@'
    #     password = '123456'
    cl = Client(addr="localhost", port=8180, username="anonymous", password="uuu@")
    cl.run()