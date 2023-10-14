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

START("13.4.4.33 - XML setChildren()");

//TEST(1, true, XML.prototype.hasOwnProperty("setChildren"));

x1 =
<alpha>
    <bravo>one</bravo>
</alpha>;

correct =
<alpha>
    <charlie>two</charlie>
</alpha>;

x1.setChildren(<charlie>two</charlie>);

TEST(2, correct, x1);

// Replace the entire contents of Jim's employee element
emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

correct =
<employees>
    <employee id="0"><name>John</name><age>35</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

emps.employee.(name == "Jim").setChildren(<name>John</name> + <age>35</age>);

TEST(3, correct, emps);

XML.prettyPrinting = false;
var xmlDoc = "<employee id='1'><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>"
var xmlList = new XMLList ("<firstname>Paul</firstname><lastname>Betlem</lastname><age>35</age>");

// propertyName as a string
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setChildren(xmlList)",
    "<employee id=\"1\"><firstname>Paul</firstname><lastname>Betlem</lastname><age>35</age></employee>",
            (MYXML = new XML(xmlDoc), MYXML.setChildren(xmlList).toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setChildren(new XML (\"<firstname>Fred</firstname>\"))",
    "<employee id=\"1\"><firstname>Fred</firstname></employee>",
            (MYXML = new XML(xmlDoc), MYXML.setChildren(new XML ("<firstname>Fred</firstname>")).toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setChildren('simple string')",
    "simple string",
             (MYXML = new XML(xmlDoc), MYXML.setChildren("simple string").toString()));


END();
