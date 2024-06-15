from ftplib import FTP

ftp = FTP()
ftp.set_debuglevel(1)
ftp.set_pasv(False)
ftp.connect(host="127.0.0.1", port=8180)

ftp.login("anonymous", "uuu@")

ftp.pwd()

ftp.mkd("test2")

ftp.cwd("test2")

ftp.pwd()

ftp.cwd("..")

ftp.rmd("test2")

ftp.retrbinary('RETR test.txt', open('test.txt', 'wb').write)

ftp.storbinary('STOR test-store.txt', open('test.txt', 'rb'))

ftp.retrlines('LIST')

ftp.quit()
