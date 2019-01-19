
function create_node(text) {
    var element = document.getElementById("comments");
    var para = document.createElement("p");
    var node = document.createTextNode(text);
    para.appendChild(node);
    element.appendChild(para);
}

fetch('/api/comments')
    .then(function(response) {
        return response.json();
    })
    .then(function(data) {
        console.log(data);
        for(var i in data)
        {
            var item = data[i];
            create_node(item.uid);
            create_node(item.text);
        }
        console.log(JSON.stringify(comments));
    });
