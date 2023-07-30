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

START("13.4.4.14 - XML hasOwnProperty()");

//TEST(1, true, XML.prototype.hasOwnProperty("hasOwnProperty"));
    
x1 =
<alpha attr1="value1">
    <bravo>one</bravo>
    <charlie>
        two
        three
        <echo>four</echo>
    </charlie>
    <delta />
</alpha>;

// Returns true for elements/attributes
TEST(2, true, x1.hasOwnProperty("bravo"));
TEST(3, true, x1.hasOwnProperty("@attr1"));
TEST(4, false, x1.hasOwnProperty("foobar"));

// Test for XML Prototype Object - returns true for XML methods.
TEST(5, true, XML.prototype.hasOwnProperty("toString"));
TEST(6, false, XML.prototype.hasOwnProperty("foobar"));

var xmlDoc = "<employee id='1'><firstname>John</firstname><lastname>Walton</lastname><age dob='1/2/1978'>25</age></employee>"

// !!@ we're not supporting prototype properties
// propertyName as a string
Assert.expectEq( "XML.prototype.hasOwnProperty('copy')", true,
             (XML.prototype.hasOwnProperty('copy')));
Assert.expectEq( "XML.prototype.hasOwnProperty('hasSimpleContent')", true,
             (XML.prototype.hasOwnProperty('hasSimpleContent')));
             
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('copy')", false,
             (MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('copy')));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('hasSimpleContent')", false,
             (MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('hasSimpleContent')));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('firstname')", true,
             (MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('firstname')));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('lastname')", true,
             (MYXML = new XML(xmlDoc), MYXML.hasOwnProperty('lastname')));

END();
