var my_xml = new XML();

my_xml.onLoad = function(success) {
    if (success) {
        trace("XML has loaded!");
    } else {
        trace("XML failed to load...");
    }

    trace(my_xml);
}

my_xml.load("whataload.xml");
