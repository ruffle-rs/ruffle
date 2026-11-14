var my_document = new XML();

var test = new XMLNode(1, "test");
var test2 = new XMLNode(1, "test2");
var test3 = new XML("<test3>test3</test3>");

my_document.appendChild(test);
my_document.appendChild(test2);
my_document.appendChild(test3);

trace("// test.parentNode === my_document");
trace(test.parentNode === my_document);
trace("// test2.parentNode === my_document");
trace(test2.parentNode === my_document);
trace("// test3.parentNode === my_document");
trace(test3.parentNode === my_document);
trace("");

trace("// test.previousSibling === null");
trace(test.previousSibling === null);
trace("// test.nextSibling === test2");
trace(test.nextSibling === test2);
trace("");

trace("// test2.previousSibling === test");
trace(test2.previousSibling === test);
trace("// test2.nextSibling === test3");
trace(test2.nextSibling === test3);
trace("");

trace("// test3.previousSibling === test2");
trace(test3.previousSibling === test2);
trace("// test3.nextSibling === null");
trace(test3.nextSibling === null);
trace("");

trace("// my_document.childNodes[0] === test");
trace(my_document.childNodes[0] === test);
trace("// my_document.childNodes[1] === test2");
trace(my_document.childNodes[1] === test2);
trace("// my_document.childNodes[2] === test3");
trace(my_document.childNodes[2] === test3);
trace("");
