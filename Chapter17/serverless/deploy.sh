set -e

extract() {
    echo "$DATA" | grep $1 | cut -d " " -f2
}

deploy() {
    echo "ASSETS DOWNLOADING"
    curl -L https://api.github.com/repos/aws-samples/aws-serverless-workshops/tarball \
        | tar xz --directory assets --wildcards "*/WebApplication/1_StaticWebHosting/website" --strip-components=4
    echo "LAMBDAS BUILDING"
    sls deploy
    echo "ASSETSi UPLOADING"
    sls client deploy
    echo "CONFIGURATION UPLOADING"
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
    echo "ASSETS REMOVING"
    sls client remove
    echo "LAMBDAS REMOVING"
    sls remove
    echo "ASSETS CLEANUP"
    rm -rf assets
    mkdir assets
    touch assets/.gitkeep
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
