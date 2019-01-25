curl -L https://api.github.com/repos/aws-samples/aws-serverless-workshops/tarball \
    | tar xz --directory assets --wildcards "*/WebApplication/1_StaticWebHosting/website" --strip-components=4
