// Compile with:
//  mtasc -main -header 200:150:30 Test.as -swf test.swf 
class Test {
    static function main(current) {

var xml;
xml = new XML("<data>A &amp; &#39; B</data>");
trace(xml.firstChild.firstChild.nodeValue);

xml = new XML("<data label=\"A &amp; &#39; B\"></data>");
trace("");
trace(xml.firstChild.attributes.label);

xml = new XML("<data>A & &thing; B</data>");
trace("");
trace(xml.firstChild.firstChild.nodeValue);

xml = new XML("<data label=\"A & &thing; B\"></data>");
trace("");
trace(xml.firstChild.attributes.label);

    }
}