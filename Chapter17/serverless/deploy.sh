set -e

extract() {
    echo "$DATA" | grep $1 | cut -d " " -f2
}

deploy() {
    sls deploy
    sls client deploy
    DATA=`sls info -v`
    POOL_ID=`extract PoolId`
    POOL_CLIENT_ID=`extract PoolClientId`
    REGION=`extract region`
    ENDPOINT=`extract ServiceEndpoint`

    CONFIG="
    window._config = {
        cognito: {
            userPoolId: '$POOL_ID',
            userPoolClientId: '$POOL_CLIENT_ID',
            region: '$REGION'
        },
        api: {
            invokeUrl: '$ENDPOINT'
        }
    };
    "

    echo "$CONFIG" | aws s3 cp - s3://rust-sls-aws/js/config.js
    INDEX=`extract BucketURL`
    echo "INDEX: $INDEX"
}

update() {
    sls deploy
}

remove() {
    sls client remove
    sls remove
}

case $1 in
    "deploy")
        deploy
        ;;
    "update")
        update
        ;;
    "remove")
        remove
        ;;
    *)
        echo "Available subcommands: deploy | remove"
        ;;
esac
