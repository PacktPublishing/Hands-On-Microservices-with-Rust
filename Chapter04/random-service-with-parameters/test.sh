function send_request() {
    echo -ne "- - - - - - - - - \nRequest: $1\nResponse: "
    curl --header "Content-Type: application/json" --request POST \
         --data "$1" \
         http://localhost:8080/random
    echo ""
}

send_request '{"distribution": "uniform", "parameters": {"start": 1, "end": 99}}'
