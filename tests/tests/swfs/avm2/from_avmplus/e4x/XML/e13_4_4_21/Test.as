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

START("13.4.4.21 - XML localName()");

//TEST(1, true, XML.prototype.hasOwnProperty("localName"));

x1 = new XML("<alpha><bravo>one</bravo><charlie><bravo>two</bravo></charlie></alpha>");
var y1;
y1 = x1.localName();
TEST(2, "string", typeof(y1));
TEST(3, "alpha", y1);

y1 = x1.bravo.localName();
x1.bravo.setNamespace("http://someuri");
TEST(4, "bravo", y1);

x1 =
<foo:alpha xmlns:foo="http://foo/">
    <foo:bravo name="bar" foo:value="one">one</foo:bravo>
</foo:alpha>;

ns = new Namespace("http://foo/");
y1 = x1.ns::bravo.localName();
TEST(5, "string", typeof(y1));
TEST(6, "bravo", y1);

y1 = x1.ns::bravo.@name.localName();
TEST(7, "name", y1);

y1 = x1.ns::bravo.@ns::value.localName();
TEST(8, "value", y1);

var xmlDoc = "<company xmlns:printer='http://colors.com/printer/'><printer:employee id='1'><name>John</name> <city>California</city> </printer:employee></company>";

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.localName()",
    "company",
    (MYXML = new XML(xmlDoc), MYXML.localName()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.localName() instanceof QName",
    false,
    (MYXML = new XML(xmlDoc), MYXML.localName() instanceof QName));

// !!@ fails in Rhino??
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.localName() instanceof String",
    true,
    (MYXML = new XML(xmlDoc), MYXML.localName() instanceof String));

Assert.expectEq( "MYXML = new XML(xmlDoc), typeof(MYXML.localName())",
    "string",
    (MYXML = new XML(xmlDoc), typeof(MYXML.localName())));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.children()[0].localName()",
    "employee",
    (MYXML = new XML(xmlDoc), MYXML.children()[0].localName()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.children()[0].localName() instanceof QName",
    false,
    (MYXML = new XML(xmlDoc), MYXML.children()[0].localName() instanceof QName));

// !!@ fails in Rhino??
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.children()[0].localName() instanceof String",
    true,
    (MYXML = new XML(xmlDoc), MYXML.children()[0].localName() instanceof String));

Assert.expectEq( "MYXML = new XML(xmlDoc), typeof(MYXML.children()[0].localName())",
    "string",
    (MYXML = new XML(xmlDoc), typeof(MYXML.children()[0].localName())));

END();
