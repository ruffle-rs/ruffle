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

START("13.5.4.6 - XMLList comments()");

//TEST(1, true, XMLList.prototype.hasOwnProperty("comments"));

// !!@ very strange that this does find comments in the list but only in elements of the list

var xmlDoc = "<!-- This is Comment --><employee id='1'><!-- This is another Comment --><name>John</name> <city>California</city> </employee><!-- me too -->";

XML.prettyPrinting = true;
XML.ignoreComments = false;
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XMLList(xmlDoc), MYXML.comments().toString()",
    "<!-- This is another Comment -->",
    (XML.ignoreComments = false, MYXML = new XMLList(xmlDoc), MYXML.comments().toString()));
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XMLList(xmlDoc), MYXML.comments() instanceof XMLList", true, (MYXML = new XMLList(xmlDoc), MYXML.comments() instanceof XMLList));
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XMLList(xmlDoc), MYXML.comments() instanceof XML", false, (MYXML = new XMLList(xmlDoc), MYXML.comments() instanceof XML));
Assert.expectEq( "XML.ignoreComments = false, MYXML = new XMLList(xmlDoc), MYXML.comments().length()", 1, (MYXML = new XMLList(xmlDoc), MYXML.comments().length() ));

Assert.expectEq( "XML.ignoreComments = false, MYXML = new XMLList(xmlDoc), XML.ignoreComments = true, MYXML.comments().toString()",
    "<!-- This is another Comment -->",
    (XML.ignoreComments = false, MYXML = new XMLList(xmlDoc), XML.ignoreComments = true, MYXML.comments().toString()));

END();
