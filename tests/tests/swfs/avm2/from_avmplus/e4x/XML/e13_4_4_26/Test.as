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

START("13.4.4.26 - XML normalize()");

//TEST(1, true, XML.prototype.hasOwnProperty("normalize"));

XML.ignoreWhitespace = false;
XML.prettyPrinting = false;

x1 =
<alpha> <bravo> one </bravo> </alpha>;

TEST_XML(2, "<alpha> <bravo> one </bravo> </alpha>", x1);
x1.normalize();
TEST_XML(3, "<alpha><bravo> one </bravo></alpha>", x1);

x1 =
<alpha>
    <bravo> one </bravo>
</alpha>;

x1.normalize();
TEST_XML(5, "<alpha><bravo> one </bravo></alpha>", x1);

XML.prettyPrinting = true;
var xml = new XML("<XML></XML>");
var a = new XML("zero");
var b = new XML("one");
var c = new XML("two");
var d = new XML("<foo>two</foo>");

xml.appendChild (a);
xml.appendChild (b);
xml.appendChild (c);
xml.appendChild (d);

Assert.expectEq( "xml has multiple text nodes, xml.normalize(), xml.toString()",
    "<XML>" + NL() + "  zeroonetwo" + NL() + "  <foo>two</foo>" + NL() + "</XML>",
    (xml.normalize(), xml.toString()));

xml = new XMLList("<XML>b</XML>");
a = new XMLList("one");
b = new XMLList("two");
c = new XMLList("<c></c>");
d = new XMLList("<foo>three</foo>");

xml.appendChild (a);
xml.appendChild (b);
xml.appendChild (c);
xml.appendChild (d);

Assert.expectEq( "xml has multiple text nodes, xml.normalize(), xml.toString()",
    "<XML>" + NL() + "  bonetwo" + NL() + "  <c/>" + NL() + "  <foo>three</foo>" + NL() + "</XML>",
(xml.normalize(), xml.toString()));

END();
