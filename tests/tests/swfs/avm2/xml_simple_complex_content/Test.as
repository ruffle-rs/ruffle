package {
  import flash.display.Sprite;
  public class Test extends Sprite { }
}

// TODO: Update this test and expected output when these properties are implemented
// XML.ignoreProcessingInstructions = false;
// XML.ignoreComments = false;

var xmlAndXmlLists: Array = [
    (<a></a>).b,
    (<a><b/></a>).b,
    (<a><b x="3"/></a>).b,
    (<a><b/><b/></a>).b,
    (<a><b/>text<b/></a>).b,
    (<a><b><c/></b></a>).b,
    (<a><b><c/></b><b/></a>).b,
    (<a><b><c/><c/></b></a>).b,
    <a>text</a>,
    <a href="a"/>,
    <?instruction ?>,
    <a><?instruction ?></a>,
    <!-- comment -->,
    <a><!-- comment --></a>,
    <![CDATA[My
    Multiline
    CDATA
    ]]>,
    <a><![CDATA[My
    Multiline
    CDATA
    ]]></a>,
];

var i: int;
for (i = 0; i < xmlAndXmlLists.length; i++) {
    var which: String = "xmlAndXmlLists[" + i + "]";
    trace(which + ".hasSimpleContent(): " + xmlAndXmlLists[i].hasSimpleContent());
    trace(which + ".hasComplexContent(): " + xmlAndXmlLists[i].hasComplexContent());
    trace("");
}
