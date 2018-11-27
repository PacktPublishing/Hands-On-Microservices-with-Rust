curl --request POST \
     --data-binary "@../../media/image.jpg" \
     --output "files/resized.jpg" \
     "http://localhost:8080/resize?width=100&height=100"
