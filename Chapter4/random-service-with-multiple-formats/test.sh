function send_request() {
    echo -ne "- - - - - - - - - \nRequest: $1\nResponse ($2): "
    curl --header "Content-Type: application/json" --request POST \
         --data "$1" \
         "http://localhost:8080/random?format=$2"
    echo ""
}

send_request '{"distribution": "uniform", "parameters": {"start": -100, "end": 100}}' json
send_request '{"distribution": "uniform", "parameters": {"start": -100, "end": 100}}' cbor
send_request '{"distribution": "uniform", "parameters": {"start": -100, "end": 100}}' xml

