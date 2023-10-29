package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

import flash.xml.XMLDocument;

trace("///");
var doc = new XMLDocument('<parent><child hello="world">a single child</child></parent>');
trace("doc: " + doc);
var single = doc.firstChild.firstChild;
trace("single: " + single);
single.removeNode();
trace("/// After removal");
trace("doc: " + doc);
trace("single: " + single);
trace("single.parentNode: " + single.parentNode);
trace("single.nextSibling: " + single.nextSibling);
trace("single.previousSibling: " + single.previousSibling);

function test(index) {
  var doc = new XMLDocument('<parent><first/><second/><last/></parent>')
  trace("///")
  trace("doc: " + doc);
  var root = doc.firstChild;
  var childNodes = root.childNodes;
  var child = childNodes[index];
  trace("root.childNodes[" + index + "]: " + child);
  child.removeNode();
  trace("/// After removal");
  trace("doc: " + doc);
  trace("child: " + child);
  trace("child.parentNode: " + child.parentNode);
  trace("child.nextSibling: " + child.nextSibling);
  trace("child.previousSibling: " + child.previousSibling);

  trace("root.firstChild: " + root.firstChild);
  trace("root.lastChild: " + root.lastChild);

  for (var i = 0; i < childNodes.length; i++) {
    trace("childNodes[" + i + "]: " + childNodes[i]);
    trace("childNodes[" + i + "].previousSibling: " + childNodes[i].previousSibling);
    trace("childNodes[" + i + "].nextSibling: " + childNodes[i].nextSibling);
  }
}

test(0);
test(1);
test(2);
