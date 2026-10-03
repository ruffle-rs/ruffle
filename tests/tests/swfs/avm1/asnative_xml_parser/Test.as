var parser = _global.ASnative(300, 0);

trace("parser typeof=" + typeof parser);

function dumpNodes(nodes)
{
    trace("length=" + nodes.length);

    for (var i = 0; i < nodes.length; i++)
    {
        var node = nodes[i];

        if (typeof node == "object")
        {
            trace(
                "node[" + i + "]" +
                " type=" + node.type +
                " value=[" + node.value + "]" +
                " empty=" + node.empty
            );

            trace("  attrs typeof=" + typeof node.attrs);

            if (typeof node.attrs == "object")
            {
                trace("  attr a=[" + node.attrs.a + "]");
                trace("  attr b=[" + node.attrs.b + "]");
                trace("  attr id=[" + node.attrs.id + "]");
                trace("  attr name=[" + node.attrs.name + "]");
            }
        }
        else
        {
            trace(
                "node[" + i + "]=[" + node + "]" +
                " typeof=" + typeof node
            );
        }
    }
}

function runCase(label, xml, ignoreWhite)
{
    trace("");
    trace("=== " + label + " ===");

    var nodes = [];
    var status = parser.call(undefined, xml, nodes, ignoreWhite);

    trace("status=" + status);
    dumpNodes(nodes);
}


// Basic parsing

runCase(
    "simple",
    "<a>EN</a>",
    false
);

runCase(
    "nested",
    "<config><core><lang>EN</lang><country>US</country></core></config>",
    false
);

runCase(
    "self closing",
    "<root><item /></root>",
    false
);

runCase(
    "mixed text",
    "<root>before<item />after</root>",
    false
);


// Attributes

runCase(
    "attributes",
    '<root><item id="5" name="test">value</item></root>',
    false
);

runCase(
    "multiple attributes",
    '<root><item a="1" b="2" /></root>',
    false
);

runCase(
    "duplicate attribute",
    '<root a="1" a="2"></root>',
    false
);

runCase(
    "attribute entities",
    '<root><item a="A &amp; B" b="&lt;x&gt;" /></root>',
    false
);


// Text and entities

runCase(
    "text entities",
    "<root>A &amp; B &lt; C &gt; D</root>",
    false
);

runCase(
    "numeric entities",
    "<root>&#65;&#x42;</root>",
    false
);

runCase(
    "unicode",
    "<root>مرحبا 世界</root>",
    false
);


// CDATA / comments / special XML nodes

runCase(
    "cdata",
    "<root><![CDATA[hello <world> & stuff]]></root>",
    false
);

runCase(
    "comment",
    "<root><!-- ignored --><item>value</item></root>",
    false
);

runCase(
    "xml declaration",
    '<?xml version="1.0"?><root></root>',
    false
);

runCase(
    "doctype",
    '<!DOCTYPE root><root></root>',
    false
);

runCase(
    "processing instruction",
    '<root><?test hello?></root>',
    false
);


// Whitespace

runCase(
    "whitespace false",
    "<root>   <item> x </item>   </root>",
    false
);

runCase(
    "whitespace true",
    "<root>   <item> x </item>   </root>",
    true
);


// Permissive structure parsing

runCase(
    "mismatched end",
    "<root><item></root>",
    false
);

runCase(
    "unmatched end",
    "<root></item></root>",
    false
);

runCase(
    "only unmatched end",
    "</root>",
    false
);

runCase(
    "unclosed element",
    "<root><item>",
    false
);


// Parse errors

runCase(
    "unquoted attribute",
    "<root a=test></root>",
    false
);

runCase(
    "unterminated attribute",
    '<root a="test></root>',
    false
);

runCase(
    "unterminated comment",
    "<root><!-- comment</root>",
    false
);

runCase(
    "unterminated cdata",
    "<root><![CDATA[test</root>",
    false
);


// Existing output array must be replaced

trace("");
trace("=== existing output array ===");

var existing = ["zero", "one", "two"];

trace("before:");
dumpNodes(existing);

var existingStatus = parser.call(
    undefined,
    "<root><item /></root>",
    existing,
    false
);

trace("status=" + existingStatus);

trace("after:");
dumpNodes(existing);


// Third argument is required

trace("");
trace("=== omitted third argument ===");

var omitted = [];

var omittedStatus = parser.call(
    undefined,
    "<root>   <item> x </item>   </root>",
    omitted
);

trace("status=" + omittedStatus);
dumpNodes(omitted);

runCase(
    "bare unclosed tag",
    "<root",
    false
);

runCase(
    "attribute missing value",
    "<root a=></root>",
    false
);

runCase(
    "attribute missing equals",
    '<root a "x"></root>',
    false
);

runCase(
    "attribute missing quote close",
    '<root a="x',
    false
);

runCase(
    "empty input",
    "",
    false
);

runCase(
    "issue 2470 malformed nesting",
    "<test><br>foo</test>",
    false
);
