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

START("13.4.4.25 - XML nodeKind()");

//TEST(1, true, XML.prototype.hasOwnProperty("nodeKind"));

x1 =
<alpha attr1="value1">
    <bravo>one</bravo>
</alpha>;

TEST(2, "element", x1.bravo.nodeKind());
TEST(3, "attribute", x1.@attr1.nodeKind());

// Non-existant node type is text
x1 = new XML();
TEST(4, "text", x1.nodeKind());
//TEST(5, "text", XML.prototype.nodeKind());

var xmlDoc = "<company><employee id='1'><name1>John</name1> <city>California</city> </employee></company>";


Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.nodeKind()",
    "element",
    (MYXML = new XML(xmlDoc), MYXML.nodeKind()));

Assert.expectEq( "MYXML = new XML(null), MYXML.nodeKind()",
    "text",
    (MYXML = new XML(null), MYXML.nodeKind()));

Assert.expectEq( "MYXML = new XML(undefined), MYXML.nodeKind()",
    "text",
    (MYXML = new XML(undefined), MYXML.nodeKind()));

Assert.expectEq( "MYXML = new XML(), MYXML.nodeKind()",
    "text",
    (MYXML = new XML(), MYXML.nodeKind()));

Assert.expectEq( "MYXML = new XML(), MYXML.children()[0].nodeKind()",
    "element",
    (MYXML = new XML(xmlDoc), MYXML.children()[0].nodeKind()));

Assert.expectEq( "MYXML = new XML(), MYXML.children()[0].attributes()[0].nodeKind()",
    "attribute",
    (MYXML = new XML(xmlDoc), MYXML.children()[0].attributes()[0].nodeKind()));

Assert.expectEq( "MYXML = new XML(), MYXML.children()[0].name1.children()[0].nodeKind()",
    "text",
    (MYXML = new XML(xmlDoc), MYXML.children()[0].name1.children()[0].nodeKind()));

XML.ignoreComments = false
Assert.expectEq( "MYXML = new XML(\"<!-- this is a comment -->\"), MYXML.nodeKind()",
    "element",
    (MYXML = new XML("<XML><!-- this is a comment --></XML>"), MYXML.nodeKind()));

Assert.expectEq( "MYXML = new XML(\"<!-- this is a comment -->\"), MYXML.children()[0].nodeKind()",
    "comment",
    (MYXML = new XML("<XML><!-- this is a comment --></XML>"), MYXML.children()[0].nodeKind()));

XML.ignoreProcessingInstructions = false
Assert.expectEq( "MYXML = new XML(\"<XML><?foo this is a pi ?></XML>\"), MYXML.nodeKind()",
    "element",
    (MYXML = new XML("<XML><?foo-- this is a pi--?></XML>"), MYXML.nodeKind()));

Assert.expectEq( "MYXML = new XML(\"<XML><?foo this is a pi ?></XML>\"), MYXML.children()[0].nodeKind()",
    "processing-instruction",
    (MYXML = new XML("<XML><?foo this is a pi ?></XML>"), MYXML.children()[0].nodeKind()));


END();
