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

START("13.4.4.37 - XML text()");

//TEST(1, true, XML.prototype.hasOwnProperty("text"));

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>
        <bravo>two</bravo>
    </charlie>
</alpha>;

TEST_XML(2, "one", x1.bravo.text());

correct = new XMLList();
correct += new XML("one");
correct += new XML("two");
TEST(3, correct, x1..bravo.text());
TEST_XML(4, "", x1.charlie.text());
TEST_XML(5, "", x1.foobar.text());
TEST_XML(6, "one", x1.*.text());

XML.prettyPrinting = false;
var xmlDoc = "<employee id='1'>foo<firstname>John</firstname>bar<lastname>Walton</lastname>still<age>25</age>reading</employee>"

// propertyName as a string
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.text().toString()",
    "foobarstillreading",
    (MYXML = new XML(xmlDoc), MYXML.text().toString()));
    
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.text() instanceof XMLList",
    true,
    (MYXML = new XML(xmlDoc), MYXML.text() instanceof XMLList));

Assert.expectEq( "MYXML = new XML('<XML></XML>'), MYXML.text().toString()",
    "",
    (MYXML = new XML('<XML></XML>'), MYXML.text().toString()));
    
Assert.expectEq( "MYXML = new XML('<XML></XML>'), MYXML.text() instanceof XMLList",
    true,
    (MYXML = new XML('<XML></XML>'), MYXML.text() instanceof XMLList));
    
xmlDoc = <a>b</a>;

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.text().toString()",
    "b",
    (MYXML = new XML(xmlDoc), MYXML.text().toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.text().toString()",
    "b",
    (MYXML = new XML(xmlDoc), MYXML.setName('c'), MYXML.text().toString()));

END();
