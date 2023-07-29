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

START("13.4.4.9 - XML comments()");

//TEST(1, true, XML.prototype.hasOwnProperty("comments"));

XML.ignoreComments = false;

x1 = new XML("<alpha><!-- comment one --><bravo><!-- comment two -->some text</bravo></alpha>");

TEST_XML(2, "<!-- comment one -->", x1.comments());
TEST_XML(3, "<!-- comment two -->", x1..*.comments());

x2 = new XML("<alpha><!-- comment one --><!-- comment 1.5 --><bravo><!-- comment two -->some text<charlie><!-- comment three --></charlie></bravo></alpha>");

TEST(4, "<!-- comment one -->\n<!-- comment 1.5 -->", x2.comments().toXMLString());
TEST(5, "<!-- comment two -->\n<!-- comment three -->", x2..*.comments().toXMLString());

XML.ignoreComments = true;
var xmlDoc = "<company><!-- This is Comment --><employee id='1'><!-- This is another Comment --><name>John</name> <city>California</city> </employee><!-- me too --></company>";

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.comments().toString()", "", (MYXML = new XML(xmlDoc), MYXML.comments().toString()));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.comments() instanceof XMLList", true, (MYXML = new XML(xmlDoc), MYXML.comments() instanceof XMLList));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.comments() instanceof XML", false, (MYXML = new XML(xmlDoc), MYXML.comments() instanceof XML));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.comments().length()", 0, (MYXML = new XML(xmlDoc), MYXML.comments().length() ));

XML.prettyPrinting = true;
XML.ignoreComments = false;
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XML(xmlDoc), MYXML.comments().toString()",
    "<!-- This is Comment -->\n<!-- me too -->",
    (XML.ignoreComments = false, MYXML = new XML(xmlDoc), MYXML.comments().toXMLString()));
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XML(xmlDoc), MYXML.comments() instanceof XMLList", true, (MYXML = new XML(xmlDoc), MYXML.comments() instanceof XMLList));
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XML(xmlDoc), MYXML.comments() instanceof XML", false, (MYXML = new XML(xmlDoc), MYXML.comments() instanceof XML));
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XML(xmlDoc), MYXML.comments().length()", 2, (MYXML = new XML(xmlDoc), MYXML.comments().length() ));

Assert.expectEq( "XML.ignoreComments = false, MYXML = new XML(xmlDoc), XML.ignoreComments = true, MYXML.comments().toString()",
    "<!-- This is Comment -->\n<!-- me too -->",
    (XML.ignoreComments = false, MYXML = new XML(xmlDoc), XML.ignoreComments = true, MYXML.comments().toXMLString()));

END();
