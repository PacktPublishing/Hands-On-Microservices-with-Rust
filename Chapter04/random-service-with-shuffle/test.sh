function send_request() {
    echo -ne "- - - - - - - - - \nRequest: $1\nResponse: "
    curl --header "Content-Type: application/json" --request POST \
         --data "$1" \
         http://localhost:8080/random
    echo ""
}

send_request '{"distribution": "uniform", "parameters": {"start": -100, "end": 100}}'
send_request '{"distribution": "shuffle", "parameters": { "data": "MTIzNDU2Nzg5MA==" } }'
send_request '{"distribution": "color", "parameters": { "from": "black", "to": "#EC670F" } }'
send_request '{"distribution": "gamma", "parameters": { "shape": 2.0, "scale": 5.0 } }'

