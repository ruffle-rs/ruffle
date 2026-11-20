function trace_idmap(idmap) {
    var keys = [];

    for (var x in idmap) {
        //Insertion sort because this test was written before we implemented Array.sort
        for (var i = 0; i <= keys.length; i += 1) {
            if (i === keys.length) {
                keys.push(x);
                break;
            } else if (x < keys[i]) {
                keys.splice(i, 0, x);
                break;
            }
        }
    }

    for (var i = 0; i < keys.length; i += 1) {
        trace("idmap has " + keys[i]);
    }
}

var my_xml = new XML("<test id='test'><test2 id='test2'>Text Node</test2> More Text</test>");

trace(my_xml.idMap.test === my_xml.childNodes[0]);
trace(my_xml.idMap.test2 === my_xml.childNodes[0].childNodes[0]);

var oldmap = my_xml.idMap;
trace_idmap(oldmap);

trace("Manually added new node via appendChild");
var next_xml = new XMLNode(1, "test3");
next_xml.attributes.id = "test3";

my_xml.appendChild(next_xml);

trace("Checking old copy of IDMAP...");
trace_idmap(oldmap);
trace("Checking new copy of IDMAP...");
trace_idmap(my_xml.idMap);

my_xml.parseXML("<test4 id='test4'>Another Test</test4>");

trace("Parsed another node with parseXML...");
trace_idmap(my_xml.idMap);

oldmap.test124 = "trace";

trace("Checking for IDMAP expandos...");
trace_idmap(my_xml.idMap);

trace(my_xml.idMap.test124);

next_xml.__proto__ = XML.prototype;
