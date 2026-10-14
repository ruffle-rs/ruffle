// NOTE: This test will not compile with an ordinary AS compiler due to the use
// of a lazy Multiname in dumpDict! To compile it, first replace the name on
// line 52 with a static Multiname, then edit the lazy Multiname back in in a
// p-code editor like the one that FFDEC provides.

package {
    import flash.display.MovieClip;
    import flash.utils.Dictionary;

    public class Testa extends MovieClip {
        public function Testa() {
            super();
            doTest();
        }

        public function doTest() {
            var dict:Dictionary = new Dictionary();
            trace("dict = new Dictionary();");
            var namePubNs:QName = new QName("","name");
            trace("namePubNs = new QName(\"\",\"name\");");
            var nameWithNs:QName = new QName("withns","name2");
            trace("nameWithNs = new QName(\"withns\",\"name2\");");

            dumpDict(dict, namePubNs, nameWithNs);

            dict[namePubNs] = "value";
            trace("dict[namePubNs] = \"value\";");

            try {
                dict[nameWithNs] = "value";
                trace("dict[nameWithNs] = \"value\";");
            } catch(e:Error) {
                trace("(couldn\'t write to dict[nameWithNs]): " + e);
            }

            dumpDict(dict, namePubNs, nameWithNs);
        }
        
        public function dumpDict(dict:Dictionary, namePubNs:QName, nameWithNs:QName) {
            trace("");
            trace("Dictionary data dump start");
            trace("  dict[namePubNs] == " + dict[namePubNs]);
            try {
                trace("  dict[nameWithNs] == " + dict[nameWithNs]);
            }
            catch(e:Error) {
                trace("  (couldn\'t read from dict[nameWithNs]): " + e);
            }
            trace("  dict[\"name\"] == " + dict["name"]);
            trace("  dict[\"name2\"] == " + dict["name2"]);
            try {
                trace("  dict[\"withns\"::\"name2\"] == " + dict.new Namespace("withns")::["name2"]);
            }
            catch(e:Error) {
                trace("  (couldn\'t access dict[\"withns\"::\"name2\"]): " + e);
            }
            trace("");
            trace("  \"name\" in dict: " + ("name" in dict));
            trace("  \"name2\" in dict: " + ("name2" in dict));
            trace("  namePubNs in dict: " + (namePubNs in dict));
            trace("  nameWithNs in dict: " + (nameWithNs in dict));
            trace("  Dictionary key enumeration");
            for(var key in dict) {
                trace("    Found a key: \"" + key + "\"");
                trace("      typeof key == " + typeof key);
                trace("      key class is " + getQualifiedClassName(key));
            }
            trace("  delete dict[\"name\"] == " + delete dict.name);
            trace("  Dictionary key enumeration (again)");
            for(var key in dict) {
                trace("    Found a key: \"" + key + "\"");
                trace("      typeof key == " + typeof key);
                trace("      key class is " + getQualifiedClassName(key));
            }
            trace("  delete dict[namePubNs] == " + delete dict[namePubNs]);
            trace("  Dictionary key enumeration (again again)");
            for(var key in dict) {
                trace("    Found a key: \"" + key + "\"");
                trace("      typeof key == " + typeof key);
                trace("      key class is " + getQualifiedClassName(key));
            }
            trace("Dictionary data dump end");
            trace("");
        }
    }
}
