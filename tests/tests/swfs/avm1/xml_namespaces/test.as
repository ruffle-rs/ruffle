function traverse(node) {
    trace("// node.nodeName");
    trace(node.nodeName);
    trace("// node.prefix");
    trace(node.prefix);
    trace("// node.localName");
    trace(node.localName);
    trace('// node.getNamespaceForPrefix("test")');
    trace(node.getNamespaceForPrefix("test"));
    trace('// node.getPrefixForNamespace("https://example.com")');
    trace(node.getPrefixForNamespace("https://example.com"));
    trace('// node.getNamespaceForPrefix("test2")');
    trace(node.getNamespaceForPrefix("test2"));
    trace('// node.getPrefixForNamespace("https://example.com/2")');
    trace(node.getPrefixForNamespace("https://example.com/2"));
    trace("// node.namespaceURI");
    trace(node.namespaceURI);
    trace("");

    for (var i = 0; i < node.childNodes.length; i++) {
        traverse(node.childNodes[i]);
    }
}

var xml = new XML(
    "<test1 xmlns='https://example.com'></test1>" +
    "<test2 xmlns:='https://example.com/colon'></test2>" +
    "<test3 xmlnsaaa='https://example.com'></test3>" +
    "<test4 xmlns='https://example.com' xmlns:='https://example.com/2'></test4>" +
    "<test5 xmlns:='https://example.com/2' xmlns='https://example.com'></test5>" +
    "<test6 xmlns='https://example.com' xmlnsaaa='https://example.com/2'></test6>" +
    "<test7 xmlnsaaa='https://example.com/2' xmlns='https://example.com'></test7>" +
    "<test8 xmlns:='https://example.com' xmlnsaaa='https://example.com/2'></test8>" +
    "<test9 xmlnsaaa='https://example.com/2' xmlns:='https://example.com'></test9>" +

    "<test:test10 xmlns:test='https://example.com'>" +
        "<test:test11 xmlns:test2='https://example.com/2'></test:test11>" +
    "</test:test10>"
);
traverse(xml);
