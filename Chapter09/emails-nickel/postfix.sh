docker run -it --rm --name test-smtp -p 2525:25  \
       -e SMTP_SERVER=smtp.example.com \
       -e SMTP_USERNAME=foo@example.com \
       -e SMTP_PASSWORD=XXXXXXXX \
       -e SERVER_HOSTNAME=smtp.example.com \
       juanluisbaptiste/postfix
