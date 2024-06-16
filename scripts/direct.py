from ftplib import FTP
import time
import threading

ftp = FTP()
ftp.set_debuglevel(1)
ftp.set_pasv(False)
ftp.connect(host="127.0.0.1", port=8180)

ftp.login("anonymous", "uuu@")

ftp.pwd()

ftp.cwd("/")

ftp.nlst()

ftp.pwd()

ftp.mkd("test2")

ftp.cwd("test2")

ftp.pwd()

ftp.cwd("..")

ftp.rmd("test2")

def delay_abort():
    print("Abort in 2.5 seconds")
    time.sleep(2.5)
    ftp.abort()
    
# threading.Thread(target=delay_abort, args=()).start()

ftp.retrbinary('RETR test.txt', open('test.txt', 'wb').write)

# threading.Thread(target=delay_abort, args=()).start()

ftp.storbinary('STOR test-store.txt', open('test-store.txt', 'rb'))

# ftp.storbinary('APPE test-store.txt', open('test-store.txt', 'rb'))

ftp.retrlines('LIST test1.txt')

ftp.retrlines('LIST')

ftp.quit()
