function debug_node(x) {
    trace(x.nodeType);
    trace(x.nodeName);
    trace(x.nodeValue);
    trace(x.localName);
    trace(x.prefix);
}

function debug_children(x) {
    var children = x.childNodes;

    trace("Children:");

    for (var i = 0; i < children.length; i += 1) {
        trace("Child " + i);
        debug_node(children[i]);
        debug_children(children[i]);
    }

    trace("End children.");
}

var x = new XML("<test></test>");

debug_node(x);
debug_children(x);
