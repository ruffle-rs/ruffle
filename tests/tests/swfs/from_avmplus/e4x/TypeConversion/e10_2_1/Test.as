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

START("10.2.1 - XML.toXMLString");

// text

var x1  = new XML("abc & 123");
var x0 = x1.toXMLString();
var y0 = "abc &amp; 123";

Assert.expectEq( "ToXMLString(text)                          :", true, (x0==y0) );

/*---------------------------------------------------------------------------*/
// comment

x1  = new XML("<x><!-- Hello World --></x>");
x0 = x1.toXMLString();
y0 = "<x/>";

Assert.expectEq( "ToXMLString(comment)                       :", true, (x0==y0) );

/*---------------------------------------------------------------------------*/
// processing instruction

x1  = new XML("<?xml version='1.0'?><x>i</x>");
x0 = x1.toXMLString();
y0 = "<x>i</x>";

Assert.expectEq( "ToXMLString(processing-instruction)        :", true, (x0==y0) );

/*---------------------------------------------------------------------------*/
// ToXMLString ( x )

XML.ignoreWhitespace = true;

x1 = new XML("<a><b>B</b><c>C</c></a>");

x0 = x1.toXMLString();
y0 = "<a>\n  <b>B</b>\n  <c>C</c>\n</a>";

Assert.expectEq( "ToXMLString(XML)                           :", true, (x0==y0) );

END();
