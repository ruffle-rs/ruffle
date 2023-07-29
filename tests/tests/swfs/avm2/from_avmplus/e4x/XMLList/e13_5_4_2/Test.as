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

START("13.5.4.2 - XMLList attribute()");

//TEST(1, true, XMLList.prototype.hasOwnProperty("attribute"));

// Test with list of size 0
emps = new XMLList();
TEST(2, "xml", typeof(emps.attribute("id")));
TEST_XML(3, "", emps.attribute("id"));

// Test with list of size 1
emps += <employee id="0"><name>Jim</name><age>25</age></employee>;

TEST(4, "xml", typeof(emps.attribute("id")));
TEST_XML(5, 0, emps.attribute("id"));

// Test with list of size > 1
emps += <employee id="1"><name>Joe</name><age>20</age></employee>;
TEST(6, "xml", typeof(emps.attribute("id")));

correct = new XMLList();
correct += new XML("0");
correct += new XML("1");
TEST(7, correct, emps.attribute("id"));

// Test one that doesn't exist - should return empty XMLList
TEST(8, "xml", typeof(emps.attribute("foobar")));

// Test args of null and undefined
try {
  emps.attribute(null);
  SHOULD_THROW(9);
} catch (ex1) {
  TEST(9, "TypeError", ex1.name);
}

try {
  emps.attribute(undefined);
  SHOULD_THROW(10);
} catch (ex2) {
  TEST(10, "TypeError", ex2.name);
}

var xmlDoc = "<TEAM foo = 'bar' two='second'>Giants</TEAM><CITY two = 'third'>Giants</CITY>";

// verify that attribute correct finds one attribute node
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('foo') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('foo') instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('foo') instanceof XML", false,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('foo') instanceof XML ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('foo').length()", 1,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('foo').length() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('foo').toString()", "bar",
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('foo').toString() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('foo')[0].nodeKind()", "attribute",
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('foo')[0].nodeKind() ));

// verify that attribute correct finds attribute nodes across children
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('two') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('two') instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('two') instanceof XML", false,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('two') instanceof XML ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('two').length()", 2,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('two').length() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('two').toString()", "secondthird",
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('two').toString() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('two')[0].nodeKind()", "attribute",
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('two')[0].nodeKind() ));

// verify that attribute doesn't find non-existent names
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XML", false,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XML ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST').length()", 0,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST').length() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST').toString()", "",
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('DOESNOTEXIST').toString() ));
             
// verify that attribute doesn't find child node names
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM') instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM') instanceof XML", false,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM') instanceof XML ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM').toString()", "",
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM').toString() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM').length()", 0,
             (MYXML = new XMLList(xmlDoc), MYXML.attribute('TEAM').length() ));

END();
