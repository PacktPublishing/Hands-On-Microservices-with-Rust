
function create_node(text) {
    var element = document.getElementById("comments");
    var para = document.createElement("p");
    var node = document.createTextNode(text);
    para.appendChild(node);
    element.appendChild(para);
}

function add_item(item) {
    create_node(item.uid);
    create_node(item.text);
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
            add_item(item);
        }
        console.log(JSON.stringify(comments));
    });

var connection = new WebSocket("ws://127.0.0.1:8080/ws", "json");
connection.onmessage = function (evt) {
    var item = JSON.parse(evt.data);
    add_item(item);
};
