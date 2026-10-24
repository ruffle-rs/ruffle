var my_xml = new XML('<test attrx="" attry="attr y string"></test>');

var test_xml = my_xml.firstChild;

trace(test_xml.attributes.attrx);
trace(test_xml.attributes.attry);
trace(test_xml.attributes.attrz);

for (var key in test_xml.attributes) {
    trace(key);
}
