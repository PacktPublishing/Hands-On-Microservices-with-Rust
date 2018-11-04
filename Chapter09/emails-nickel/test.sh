if [ -z "$1" ]; then
    echo "email argument not set"
    exit 1
fi

curl -d "to=$1&code=passcode" -X POST http://localhost:7000/send
