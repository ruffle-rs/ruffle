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

// Parser statuses and malformed input
runCase("valid nested XML; ignoreWhite=false", "<a><b>foo</b></a>", false);
runCase("valid nested XML; ignoreWhite=true", "<a><b>foo</b></a>", true);
runCase("unterminated CDATA after start tag; ignoreWhite=false", "<a><![CDATA[foo</a>", false);
runCase("unterminated CDATA after start tag; ignoreWhite=true", "<a><![CDATA[foo</a>", true);
runCase("unterminated XML declaration; ignoreWhite=false", "<?xml version=\"1.0\"", false);
runCase("unterminated XML declaration; ignoreWhite=true", "<?xml version=\"1.0\"", true);
runCase("unterminated DOCTYPE; ignoreWhite=false", "<!DOCTYPE foo", false);
runCase("unterminated DOCTYPE; ignoreWhite=true", "<!DOCTYPE foo", true);
runCase("unterminated comment after start tag; ignoreWhite=false", "<a><!-- foo</a>", false);
runCase("unterminated comment after start tag; ignoreWhite=true", "<a><!-- foo</a>", true);
runCase("unquoted attribute with text; ignoreWhite=false", "<a b=c>foo</a>", false);
runCase("unquoted attribute with text; ignoreWhite=true", "<a b=c>foo</a>", true);
runCase("unterminated attribute with text; ignoreWhite=false", "<a b=\"foo>bar</a>", false);
runCase("unterminated attribute with text; ignoreWhite=true", "<a b=\"foo>bar</a>", true);
runCase("bare less than; ignoreWhite=false", "<", false);
runCase("bare less than; ignoreWhite=true", "<", true);
runCase("bare end tag; ignoreWhite=false", "</", false);
runCase("bare end tag; ignoreWhite=true", "</", true);
runCase("bare declaration start; ignoreWhite=false", "<?", false);
runCase("bare declaration start; ignoreWhite=true", "<?", true);
runCase("bare xml declaration; ignoreWhite=false", "<?xml", false);
runCase("bare xml declaration; ignoreWhite=true", "<?xml", true);
runCase("declaration missing question mark; ignoreWhite=false", "<?xml version=\"1.0\">", false);
runCase("declaration missing question mark; ignoreWhite=true", "<?xml version=\"1.0\">", true);
runCase("declaration without version; ignoreWhite=false", "<?xml?>", false);
runCase("declaration without version; ignoreWhite=true", "<?xml?>", true);
runCase("declaration after element; ignoreWhite=false", "<a/><?xml version=\"1.0\"?>", false);
runCase("declaration after element; ignoreWhite=true", "<a/><?xml version=\"1.0\"?>", true);
runCase("valid processing instruction; ignoreWhite=false", "<?test foo?>", false);
runCase("valid processing instruction; ignoreWhite=true", "<?test foo?>", true);
runCase("unfinished processing instruction; ignoreWhite=false", "<?test foo", false);
runCase("unfinished processing instruction; ignoreWhite=true", "<?test foo", true);
runCase("bare doctype; ignoreWhite=false", "<!DOCTYPE", false);
runCase("bare doctype; ignoreWhite=true", "<!DOCTYPE", true);
runCase("doctype without name; ignoreWhite=false", "<!DOCTYPE>", false);
runCase("doctype without name; ignoreWhite=true", "<!DOCTYPE>", true);
runCase("doctype internal subset; ignoreWhite=false", "<!DOCTYPE a [<!ELEMENT a ANY>]><a/>", false);
runCase("doctype internal subset; ignoreWhite=true", "<!DOCTYPE a [<!ELEMENT a ANY>]><a/>", true);
runCase("unfinished internal subset; ignoreWhite=false", "<!DOCTYPE a [<!ELEMENT a ANY>", false);
runCase("unfinished internal subset; ignoreWhite=true", "<!DOCTYPE a [<!ELEMENT a ANY>", true);
runCase("doctype literal unicode escapes; ignoreWhite=false", "<!DOCTYPE a SYSTEM \"\u0645\u0631\u062d\u0628\u0627.dtd\"><a/>", false);
runCase("doctype literal unicode escapes; ignoreWhite=true", "<!DOCTYPE a SYSTEM \"\u0645\u0631\u062d\u0628\u0627.dtd\"><a/>", true);
runCase("declaration literal unicode escapes; ignoreWhite=false", "<?xml version=\"1.0\" note=\"\u0645\u0631\u062d\u0628\u0627\"?><a/>", false);
runCase("declaration literal unicode escapes; ignoreWhite=true", "<?xml version=\"1.0\" note=\"\u0645\u0631\u062d\u0628\u0627\"?><a/>", true);
runCase("bare comment; ignoreWhite=false", "<!--", false);
runCase("bare comment; ignoreWhite=true", "<!--", true);
runCase("comment partial close; ignoreWhite=false", "<!--foo--", false);
runCase("comment partial close; ignoreWhite=true", "<!--foo--", true);
runCase("bare cdata; ignoreWhite=false", "<![CDATA[", false);
runCase("bare cdata; ignoreWhite=true", "<![CDATA[", true);
runCase("cdata partial close; ignoreWhite=false", "<![CDATA[foo]]", false);
runCase("cdata partial close; ignoreWhite=true", "<![CDATA[foo]]", true);
runCase("unclosed start; ignoreWhite=false", "<a>", false);
runCase("unclosed start; ignoreWhite=true", "<a>", true);
runCase("unmatched end; ignoreWhite=false", "</a>", false);
runCase("unmatched end; ignoreWhite=true", "</a>", true);
runCase("mismatched end; ignoreWhite=false", "<a></b>", false);
runCase("mismatched end; ignoreWhite=true", "<a></b>", true);
runCase("reversed nesting; ignoreWhite=false", "<a><b></a></b>", false);
runCase("reversed nesting; ignoreWhite=true", "<a><b></a></b>", true);
runCase("multiple roots; ignoreWhite=false", "<a/><b/>", false);
runCase("multiple roots; ignoreWhite=true", "<a/><b/>", true);
runCase("single quote unfinished; ignoreWhite=false", "<a b='foo", false);
runCase("single quote unfinished; ignoreWhite=true", "<a b='foo", true);
runCase("double quote unfinished; ignoreWhite=false", "<a b=\"foo", false);
runCase("double quote unfinished; ignoreWhite=true", "<a b=\"foo", true);
runCase("attribute without equals; ignoreWhite=false", "<a b \"foo\"/>", false);
runCase("attribute without equals; ignoreWhite=true", "<a b \"foo\"/>", true);
runCase("attribute without value; ignoreWhite=false", "<a b=>", false);
runCase("attribute without value; ignoreWhite=true", "<a b=>", true);
runCase("duplicate attributes; ignoreWhite=false", "<a b=\"first\" b=\"second\"/>", false);
runCase("duplicate attributes; ignoreWhite=true", "<a b=\"first\" b=\"second\"/>", true);
runCase("comment before unfinished tag; ignoreWhite=false", "<!-- <x a=\" --> <a", false);
runCase("comment before unfinished tag; ignoreWhite=true", "<!-- <x a=\" --> <a", true);
runCase("cdata before unfinished tag; ignoreWhite=false", "<![CDATA[<x a=\"]]><a", false);
runCase("cdata before unfinished tag; ignoreWhite=true", "<![CDATA[<x a=\"]]><a", true);
runCase("quoted angle bracket; ignoreWhite=false", "<a b=\"<x>\"/>", false);
runCase("quoted angle bracket; ignoreWhite=true", "<a b=\"<x>\"/>", true);
runCase("unknown entity; ignoreWhite=false", "<a>&unknown;</a>", false);
runCase("unknown entity; ignoreWhite=true", "<a>&unknown;</a>", true);
runCase("unfinished entity; ignoreWhite=false", "<a>&amp</a>", false);
runCase("unfinished entity; ignoreWhite=true", "<a>&amp</a>", true);
runCase("comment error before cdata error; ignoreWhite=false", "<!--foo <![CDATA[bar", false);
runCase("comment error before cdata error; ignoreWhite=true", "<!--foo <![CDATA[bar", true);
runCase("cdata error before comment error; ignoreWhite=false", "<![CDATA[foo <!--bar", false);
runCase("cdata error before comment error; ignoreWhite=true", "<![CDATA[foo <!--bar", true);
runCase("attribute error before comment error; ignoreWhite=false", "<a b=\"foo><!--bar", false);
runCase("attribute error before comment error; ignoreWhite=true", "<a b=\"foo><!--bar", true);
runCase("valid prefix before declaration error; ignoreWhite=false", "<a/><?xml version=\"1.0\"", false);
runCase("valid prefix before declaration error; ignoreWhite=true", "<a/><?xml version=\"1.0\"", true);
runCase("valid prefix before doctype error; ignoreWhite=false", "<a/><!DOCTYPE foo", false);
runCase("valid prefix before doctype error; ignoreWhite=true", "<a/><!DOCTYPE foo", true);

// Attribute validation, declarations, DOCTYPE, and Unicode
runCase("attribute raw less than; ignoreWhite=false", "<a b=\"<\"/>", false);
runCase("attribute raw less than; ignoreWhite=true", "<a b=\"<\"/>", true);
runCase("attribute raw greater than; ignoreWhite=false", "<a b=\">\"/>", false);
runCase("attribute raw greater than; ignoreWhite=true", "<a b=\">\"/>", true);
runCase("attribute less than in text; ignoreWhite=false", "<a b=\"x<y\"/>", false);
runCase("attribute less than in text; ignoreWhite=true", "<a b=\"x<y\"/>", true);
runCase("attribute greater than in text; ignoreWhite=false", "<a b=\"x>y\"/>", false);
runCase("attribute greater than in text; ignoreWhite=true", "<a b=\"x>y\"/>", true);
runCase("attribute raw greater single quotes; ignoreWhite=false", "<a b='x>y'/>", false);
runCase("attribute raw greater single quotes; ignoreWhite=true", "<a b='x>y'/>", true);
runCase("attribute encoded less than; ignoreWhite=false", "<a b=\"&lt;\"/>", false);
runCase("attribute encoded less than; ignoreWhite=true", "<a b=\"&lt;\"/>", true);
runCase("attribute encoded greater than; ignoreWhite=false", "<a b=\"&gt;\"/>", false);
runCase("attribute encoded greater than; ignoreWhite=true", "<a b=\"&gt;\"/>", true);
runCase("attribute encoded angle pair; ignoreWhite=false", "<a b=\"&lt;x&gt;\"/>", false);
runCase("attribute encoded angle pair; ignoreWhite=true", "<a b=\"&lt;x&gt;\"/>", true);
runCase("attribute greater then malformed attribute; ignoreWhite=false", "<a b=\">\" c=bad/>", false);
runCase("attribute greater then malformed attribute; ignoreWhite=true", "<a b=\">\" c=bad/>", true);
runCase("attribute greater then missing equals; ignoreWhite=false", "<a b=\">\" c \"bad\"/>", false);
runCase("attribute greater then missing equals; ignoreWhite=true", "<a b=\">\" c \"bad\"/>", true);
runCase("attribute malformed then greater; ignoreWhite=false", "<a c=bad b=\">\"/>", false);
runCase("attribute malformed then greater; ignoreWhite=true", "<a c=bad b=\">\"/>", true);
runCase("duplicate ignored greater; ignoreWhite=false", "<a b=\"first\" b=\">\"/>", false);
runCase("duplicate ignored greater; ignoreWhite=true", "<a b=\"first\" b=\">\"/>", true);
runCase("duplicate first greater; ignoreWhite=false", "<a b=\">\" b=\"second\"/>", false);
runCase("duplicate first greater; ignoreWhite=true", "<a b=\">\" b=\"second\"/>", true);
runCase("attribute less than unfinished; ignoreWhite=false", "<a b=\"<", false);
runCase("attribute less than unfinished; ignoreWhite=true", "<a b=\"<", true);
runCase("empty doctype followed by element; ignoreWhite=false", "<!DOCTYPE><a/>", false);
runCase("empty doctype followed by element; ignoreWhite=true", "<!DOCTYPE><a/>", true);
runCase("empty doctype between elements; ignoreWhite=false", "<a/><!DOCTYPE><b/>", false);
runCase("empty doctype between elements; ignoreWhite=true", "<a/><!DOCTYPE><b/>", true);
runCase("repeated empty doctype; ignoreWhite=false", "<!DOCTYPE><!DOCTYPE><a/>", false);
runCase("repeated empty doctype; ignoreWhite=true", "<!DOCTYPE><!DOCTYPE><a/>", true);
runCase("empty doctype followed by error; ignoreWhite=false", "<!DOCTYPE><a b=\"bad", false);
runCase("empty doctype followed by error; ignoreWhite=true", "<!DOCTYPE><a b=\"bad", true);
runCase("empty doctype followed by declaration; ignoreWhite=false", "<!DOCTYPE><?xml?><a/>", false);
runCase("empty doctype followed by declaration; ignoreWhite=true", "<!DOCTYPE><?xml?><a/>", true);
runCase("empty doctype with spaces; ignoreWhite=false", "<!DOCTYPE   ><a/>", false);
runCase("empty doctype with spaces; ignoreWhite=true", "<!DOCTYPE   ><a/>", true);
runCase("doctype extra spaces; ignoreWhite=false", "<!DOCTYPE   a><a/>", false);
runCase("doctype extra spaces; ignoreWhite=true", "<!DOCTYPE   a><a/>", true);
runCase("doctype trailing spaces; ignoreWhite=false", "<!DOCTYPE a   ><a/>", false);
runCase("doctype trailing spaces; ignoreWhite=true", "<!DOCTYPE a   ><a/>", true);
runCase("doctype lowercase; ignoreWhite=false", "<!doctype a><a/>", false);
runCase("doctype lowercase; ignoreWhite=true", "<!doctype a><a/>", true);
runCase("doctype mixed case; ignoreWhite=false", "<!DocType a><a/>", false);
runCase("doctype mixed case; ignoreWhite=true", "<!DocType a><a/>", true);
runCase("doctype quoted greater; ignoreWhite=false", "<!DOCTYPE a SYSTEM \"x>y\"><a/>", false);
runCase("doctype quoted greater; ignoreWhite=true", "<!DOCTYPE a SYSTEM \"x>y\"><a/>", true);
runCase("doctype backslash; ignoreWhite=false", "<!DOCTYPE a SYSTEM \"dir\\file.dtd\"><a/>", false);
runCase("doctype backslash; ignoreWhite=true", "<!DOCTYPE a SYSTEM \"dir\\file.dtd\"><a/>", true);
runCase("doctype unicode; ignoreWhite=false", "<!DOCTYPE a SYSTEM \"مرحبا.dtd\"><a/>", false);
runCase("doctype unicode; ignoreWhite=true", "<!DOCTYPE a SYSTEM \"مرحبا.dtd\"><a/>", true);
runCase("declaration unicode; ignoreWhite=false", "<?xml version=\"1.0\" note=\"مرحبا\"?><a/>", false);
runCase("declaration unicode; ignoreWhite=true", "<?xml version=\"1.0\" note=\"مرحبا\"?><a/>", true);
runCase("unicode names and attribute; ignoreWhite=false", "<مرحبا b=\"العالم\">世界</مرحبا>", false);
runCase("unicode names and attribute; ignoreWhite=true", "<مرحبا b=\"العالم\">世界</مرحبا>", true);
runCase("declaration uppercase unfinished; ignoreWhite=false", "<?XML version=\"1.0\"", false);
runCase("declaration uppercase unfinished; ignoreWhite=true", "<?XML version=\"1.0\"", true);
runCase("declaration uppercase complete; ignoreWhite=false", "<?XML version=\"1.0\"?>", false);
runCase("declaration uppercase complete; ignoreWhite=true", "<?XML version=\"1.0\"?>", true);
runCase("xml-prefixed PI unfinished; ignoreWhite=false", "<?xmlfoo", false);
runCase("xml-prefixed PI unfinished; ignoreWhite=true", "<?xmlfoo", true);
runCase("xml-prefixed PI complete; ignoreWhite=false", "<?xmlfoo?>", false);
runCase("xml-prefixed PI complete; ignoreWhite=true", "<?xmlfoo?>", true);
runCase("xml stylesheet PI unfinished; ignoreWhite=false", "<?xml-stylesheet href=\"x\"", false);
runCase("xml stylesheet PI unfinished; ignoreWhite=true", "<?xml-stylesheet href=\"x\"", true);
runCase("xml stylesheet PI complete; ignoreWhite=false", "<?xml-stylesheet href=\"x\"?>", false);
runCase("xml stylesheet PI complete; ignoreWhite=true", "<?xml-stylesheet href=\"x\"?>", true);
runCase("declaration tab separator unfinished; ignoreWhite=false", "<?xml\tversion=\"1.0\"", false);
runCase("declaration tab separator unfinished; ignoreWhite=true", "<?xml\tversion=\"1.0\"", true);
runCase("doctype bare after text; ignoreWhite=false", "prefix<!DOCTYPE", false);
runCase("doctype bare after text; ignoreWhite=true", "prefix<!DOCTYPE", true);
runCase("declaration bare after text; ignoreWhite=false", "prefix<?xml", false);
runCase("declaration bare after text; ignoreWhite=true", "prefix<?xml", true);
runCase("BOM before doctype; ignoreWhite=false", "﻿<!DOCTYPE a><a/>", false);
runCase("BOM before doctype; ignoreWhite=true", "﻿<!DOCTYPE a><a/>", true);
runCase("BOM before empty doctype; ignoreWhite=false", "﻿<!DOCTYPE><a/>", false);
runCase("BOM before empty doctype; ignoreWhite=true", "﻿<!DOCTYPE><a/>", true);

runCase("attribute error priority: raw less before unquoted", '<a b="<" c=bad/>', false);
runCase("attribute error priority: raw less before missing equals", '<a b="<" c "bad"/>', false);
runCase("attribute error priority: unquoted before raw less", '<a c=bad b="<"/>', false);
runCase("attribute error priority: duplicate raw less", '<a b="first" b="<"/>', false);
