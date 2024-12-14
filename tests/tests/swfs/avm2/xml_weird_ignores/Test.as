package {
    import flash.display.MovieClip;
    
    public class Test extends MovieClip {

        public function Test() {
            XML.prettyPrinting = false;
            var tests = [
                "<![CDATA[abcd]]>text",
                "<!-- comment --><elem><?pi PI?><!-- inner --> innerText </elem><?pi PI?><![CDATA[  ]]>",
                "    <![CDATA[abcd]]><body/>",
                "<body/><?pi PI?><body2/><!-- comment -->",
                "<elem><?pi innerPI?></elem><?pi PI?>",
                "    <body/>   ",
                "<?pi PI?><!-- comment -->"
            ];
            var settingss = [
                [false, false, false],
                [false, false, true],
                [false, true, false],
                [false, true, true],
                [true, false, false],
                [true, false, true],
                [true, true, false],
                [true, true, true]
            ];
            for(var i in tests) {
                var test = tests[i];
                for(var j in settingss) {
                    var settings = settingss[j];
                    XML.ignoreComments = settings[0];
                    XML.ignoreProcessingInstructions = settings[1];
                    XML.ignoreWhitespace = settings[2];
                    try
                    {
                        var x:XML = new XML(test);
                        trace(x.toXMLString());
                    }
                    catch(e:Error)
                    {
                        trace("err: " + e);
                    }
                }
            }
        }
    }
}

