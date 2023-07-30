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
import com.adobe.test.Utils;

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
 

START("13.4.4.23 - XML getNamespace()");

//TEST(1, true, XML.prototype.hasOwnProperty("getNamespace"));

// Prefix case
x1 =
<foo:alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
    <foo:bravo>one</foo:bravo>
</foo:alpha>;

//y = x1.getNamespace();
y1 = myGetNamespace(x1);
TEST(2, "object", typeof(y1));
TEST(3, Namespace("http://foo/"), y1);

//y = x1.getNamespace("bar");
y1 = myGetNamespace(x1, "bar");
TEST(4, "object", typeof(y1));
TEST(5, Namespace("http://bar/"), y1);

// No Prefix Case
x1 =
<alpha xmlns="http://foobar/">
    <bravo>one</bravo>
</alpha>;

//y = x1.getNamespace();
y1 = myGetNamespace(x1);
TEST(6, "object", typeof(y1));
TEST(7, Namespace("http://foobar/"), y1);

// No Namespace case
x1 =
<alpha>
    <bravo>one</bravo>
</alpha>;
//TEST(8, Namespace(""), x1.getNamespace());
TEST(8, Namespace(""), myGetNamespace(x1));

// Namespaces of attributes
x1 =
<alpha xmlns="http://foo/">
    <ns:bravo xmlns:ns="http://bar" name="two" ns:value="three">one</ns:bravo>
</alpha>;

var ns = new Namespace("http://bar");
//y1 = x1.ns::bravo.@name.getNamespace();
y1 = myGetNamespace(x1.ns::bravo.@name);
TEST(9, Namespace(""), y1);

//y1 = x1.ns::bravo.@ns::value.getNamespace();
y1 = myGetNamespace(x1.ns::bravo.@ns::value);
TEST(10, ns.toString(), y1.toString());

var xmlDoc = "<?xml version=\"1.0\"?><xsl:stylesheet xmlns:xsl=\"http://www.w3.org/TR/xsl\"><b><c xmlns:foo=\"http://www.foo.org/\">hi</c></b></xsl:stylesheet>";
var ns1 = Namespace ("xsl", "http://www.w3.org/TR/xsl");
var ns2 = Namespace ("foo", "http://www.foo.org");


function convertToString(o:Object){
  return o.toString();
}

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.getNamespace()",
    "http://www.w3.org/TR/xsl",
    (MYXML = new XML(xmlDoc), myGetNamespace(MYXML).toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.getNamespace()",
    convertToString(new Namespace("http://www.w3.org/TR/xsl")),
    (MYXML = new XML(xmlDoc), myGetNamespace(MYXML)).toString());

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.getNamespace('stylesheet')",
    convertToString(new Namespace ("http://www.w3.org/TR/xsl")),
    (MYXML = new XML(xmlDoc), myGetNamespace(MYXML, 'xsl')).toString());

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.getNamespace('xsl')",
    "http://www.w3.org/TR/xsl", (MYXML = new XML(xmlDoc),
    myGetNamespace(MYXML, 'xsl').toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.getNamespace('foo')",
    undefined, (MYXML = new XML(xmlDoc),
    myGetNamespace(MYXML, 'foo')));


Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.b.c.getNamespace('foo')",
    "http://www.foo.org/", (MYXML = new XML(xmlDoc),
    myGetNamespace(MYXML.b.c, 'foo').toString()));
    
    
x1 =
<><foo:alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
<foo:bravo>one</foo:bravo>
</foo:alpha><a>b</a></>;

try {
    ns = x1.namespace();
    result = ns;
} catch (e1) {
    result = Utils.typeError(e1.toString());
}

Assert.expectEq("Calling namespace on list with two items", "TypeError: Error #1086", result);

END();
