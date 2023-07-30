/* -*- Mode: java; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
public class Test {}
}

import com.adobe.test.Assert;

function START(summary)
{
      // print out bugnumber

     /*if ( BUGNUMBER ) {
              writeLineToLog ("BUGNUMBER: " + BUGNUMBER );
      }*/
    XML.setSettings (null);
    testcases = new Array();

    // text field for results
    tc = 0;
    /*this.addChild ( tf );
    tf.x = 30;
    tf.y = 50;
    tf.width = 200;
    tf.height = 400;*/

    //_print(summary);
    var summaryParts = summary.split(" ");
    //_print("section: " + summaryParts[0] + "!");
    //fileName = summaryParts[0];

}

function TEST(section, expected, actual)
{
    AddTestCase(section, expected, actual);
}
 

function TEST_XML(section, expected, actual)
{
  var actual_t = typeof actual;
  var expected_t = typeof expected;

  if (actual_t != "xml") {
    // force error on type mismatch
    TEST(section, new XML(), actual);
    return;
  }

  if (expected_t == "string") {

    TEST(section, expected, actual.toXMLString());
  } else if (expected_t == "number") {

    TEST(section, String(expected), actual.toXMLString());
  } else {
    reportFailure ("", 'Bad TEST_XML usage: type of expected is "+expected_t+", should be number or string');
  }
}

function reportFailure (section, msg)
{
  trace("~FAILURE: " + section + " | " + msg);
}

function AddTestCase( description, expect, actual ) {
   testcases[tc++] = Assert.expectEq(description, "|"+expect+"|", "|"+actual+"|" );
}

function myGetNamespace (obj, ns) {
    if (ns != undefined) {
        return obj.namespace(ns);
    } else {
        return obj.namespace();
    }
}




function NL()
{
  //return java.lang.System.getProperty("line.separator");
  return "\n";
}


function BUG(arg){
  // nothing here
}

function END()
{
    //test();
}

START("13.4.4.36 - setNamespace");

//TEST(1, true, XML.prototype.hasOwnProperty("setNamespace"));

x1 =
<foo:alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
    <foo:bravo>one</foo:bravo>
</foo:alpha>;

correct =
<bar:alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
    <foo:bravo>one</foo:bravo>
</bar:alpha>;

x1.setNamespace("http://bar/");

TEST(2, correct, x1);

XML.prettyPrinting = false;
var xmlDoc = "<xsl:stylesheet xmlns:xsl=\"http://www.w3.org/TR/xsl\"><xsl:template match=\"/\"><html>body</html></xsl:template></xsl:stylesheet>"

// !!@ Rhino comes with with the "ns" prefix for this namespace.  I have no idea how that happens.
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setNamespace('http://xyx.org/xml'),MYXML.toString()",
    "<stylesheet xmlns:xsl=\"http://www.w3.org/TR/xsl\" xmlns=\"http://xyx.org/xml\"><xsl:template match=\"/\"><html>body</html></xsl:template></stylesheet>",
    (MYXML = new XML(xmlDoc), MYXML.setNamespace('http://xyx.org/xml'), MYXML.toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setNamespace('http://xyx.org/xml'),MYXML.getNamespace()",
    "http://xyx.org/xml",
    (MYXML = new XML(xmlDoc), MYXML.setNamespace('http://xyx.org/xml'), myGetNamespace(MYXML).toString()));


xmlDoc = "<a><b><c>d</c></b></a>";
MYXML = new XML(xmlDoc);
MYXML.setNamespace('http://foo');
MYXML.b.c.setNamespace('http://bar');

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setNamespace('http://zxz.org/xml'),MYXML.toString()",
    "<a xmlns=\"http://foo\"><b><aaa:c xmlns:aaa=\"http://bar\">d</aaa:c></b></a>",
    (MYXML.toString()));
    
var n1 = new Namespace('zzz', 'http://www.zzz.com');
var n2 = new Namespace('zzz', 'http://www.zzz.org');
var n3 = new Namespace('abc', 'http://www.zzz.com');
var n4 = new Namespace('def', 'http://www.zzz.com');

xmlDoc = "<a><b>c</b></a>";

Assert.expectEq("Adding two namespaces by uri",
    "<a xmlns=\"http://www.zzz.com\"><aaa:b xmlns:aaa=\"http://www.zzz.org\">c</aaa:b></a>",
    (MYXML = new XML(xmlDoc), MYXML.setNamespace('http://www.zzz.com'), MYXML.b.setNamespace('http://www.zzz.org'), MYXML.toString()));

Assert.expectEq("Adding two namespaces with prefix 'zzz'",
    "<zzz:a xmlns:zzz=\"http://www.zzz.com\"><zzz:b xmlns:zzz=\"http://www.zzz.org\">c</zzz:b></zzz:a>",
    (MYXML = new XML(xmlDoc), MYXML.setNamespace(n1), MYXML.b.setNamespace(n2), MYXML.toString()));
    
Assert.expectEq("Adding two namespaces with prefix 'zzz'",
    "<abc:a xmlns:abc=\"http://www.zzz.com\"><abc:b xmlns:def=\"http://www.zzz.com\">c</abc:b></abc:a>",
    (MYXML = new XML(xmlDoc), MYXML.setNamespace(n3), MYXML.b.setNamespace(n4), MYXML.toString()));
    

ns = new Namespace("moo", "http://moo/");
ns2 = new Namespace("zoo", "http://zoo/");
ns3 = new Namespace("noo", "http://mootar");
ns4 = new Namespace("moo", "http://bar/");
ns5 = new Namespace("poo", "http://moo/");
x1 = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo attr="1">one</moo:bravo>
</moo:alpha>;

correct = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo zoo:attr="1" xmlns:zoo="http://zoo/">one</moo:bravo>
</moo:alpha>;

var MYXML = new XML(x1);
MYXML.ns::bravo.@attr.setNamespace(ns2);

Assert.expectEq("Calling setNamespace() on an attribute", correct.toString(), MYXML.toString());

correct = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo attr="1" xmlns="zoo">one</moo:bravo>
</moo:alpha>;

Assert.expectEq("Calling setNamespace() on an attribute with conflicting namespace prefix", correct.toString(), (MYXML = new XML(x1), MYXML.ns::bravo.@attr.setNamespace("zoo"), MYXML.toString()));

correct = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo attr="1" xmlns="moo">one</moo:bravo>
</moo:alpha>;

Assert.expectEq("Calling setNamespace() on an attribute with conflicting namespace prefix", correct.toString(), (MYXML = new XML(x1), MYXML.ns::bravo.@attr.setNamespace("moo"), MYXML.toString()));

correct = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo attr="1" xmlns="http://zoo/">one</moo:bravo>
</moo:alpha>;

Assert.expectEq("Calling setNamespace() on an attribute with conflicting namespace prefix", correct.toString(), (MYXML = new XML(x1), MYXML.ns::bravo.@attr.setNamespace("http://zoo/"), MYXML.toString()));

correct = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo noo:attr="1" xmlns:noo="http://mootar">one</moo:bravo>
</moo:alpha>;
delete correct;

Assert.expectEq("Calling setNamespace() on an attribute with conflicting namespace", correct.toString(), (MYXML = new XML(x1), MYXML.ns::bravo.@attr.setNamespace(ns3), MYXML.toString()));

var correct = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo moo:attr="1" xmlns:moo="http://bar/">one</moo:bravo>
</moo:alpha>;

Assert.expectEq("Calling setNamespace() on an attribute with conflicting namespace prefix", correct.toString(), (MYXML = new XML(x1), MYXML.ns::bravo.@attr.setNamespace(ns4), MYXML.toString()));

var correct = <moo:alpha xmlns:moo="http://moo/" xmlns:tar="http://tar/">
    <moo:bravo poo:attr="1" xmlns:poo="http://moo/">one</moo:bravo>
</moo:alpha>;

Assert.expectEq("Calling setNamespace() on an attribute with conflicting namespace", correct.toString(), (MYXML = new XML(x1), MYXML.ns::bravo.@attr.setNamespace(ns5), MYXML.toString()));


x1 = <alpha>
    <bravo attr="1">one</bravo>
</alpha>;

correct = <alpha>
    <bravo moo:attr="1" xmlns:moo="http://moo/">one</bravo>
</alpha>;

Assert.expectEq("Calling setNamespace() on an attribute with no existing namespace", correct.toString(), (MYXML = new XML(x1), MYXML.bravo.@attr.setNamespace(ns), MYXML.toString()));

    
END();
