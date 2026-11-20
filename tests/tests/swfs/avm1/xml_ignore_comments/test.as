var my_xml = new XML("<!--comment-->text<node><!--comment-->text<node><!--comment-->text</node>text<!--comment--></node>text<!--comment-->");

function dumpForwards(path, parent) {
    var child = parent.firstChild;
    path += ".firstChild";
    while (child) {
        trace(path + " = " + child);
        dumpForwards(path, child);

        child = child.nextSibling;
        path += ".nextSibling";
    }
}

function dumpBackwards(path, parent) {
    var child = parent.lastChild;
    path += ".lastChild";
    while (child) {
        trace(path + " = " + child);
        dumpBackwards(path, child);

        child = child.previousSibling;
        path += ".previousSibling";
    }
}

function dumpChildren(path, parent) {
    for (var i = 0; i < parent.childNodes.length; i++) {
        trace(path + "[" + i + "] = " + parent.childNodes[i]);
        dumpChildren(path + "[" + i + "]", parent.childNodes[i]);
    }
}

dumpForwards("my_xml", my_xml);
dumpBackwards("my_xml", my_xml);
dumpChildren("my_xml", my_xml);
