from ftplib import FTP

ftp = FTP()
ftp.set_debuglevel(1)
ftp.connect(host="127.0.0.1", port=8180)

ftp.login("anonymous", "uuu@")

ftp.pwd()

ftp.mkd("test2")

ftp.cwd("test2")

ftp.pwd()

ftp.cwd("..")

ftp.rmd("test2")

def download_file(ftp, filename):
    with open(filename, 'wb') as f:
        ftp.retrbinary('RETR ' + filename, f.write)
        
download_file(ftp, "test.txt")

ftp.retrlines('LIST')

ftp.quit()
