/* -*- Mode: java; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
public class Test {}
}

import avmplus.System
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

START("13.4.4.32 - XML replace()");

//TEST(1, true, XML.prototype.hasOwnProperty("replace"));

// Replace the first employee record with an open staff requisition
emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

correct =
<employees>
    <requisition status="open" />
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

emps.replace(0, <requisition status="open" />);

TEST(2, correct, emps);

// Replace all children with open staff requisition

emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

correct =
<employees>
    <requisition status="open" />
</employees>;

emps.replace("*", <requisition status="open" />);

TEST(3, correct, emps);

// Replace all employee elements with open staff requisition

emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

correct =
<employees>
    <requisition status="open" />
</employees>;

emps.replace("employee", <requisition status="open" />);

TEST(4, correct, emps);

XML.prettyPrinting = false;
var xmlDoc = "<employee id='1'><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>"

// propertyName as a string
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.replace(0,'Mr. John')",
    "<employee id=\"1\">Mr. John<lastname>Walton</lastname><age>25</age></employee>",
             (MYXML = new XML(xmlDoc), MYXML.replace(0,'Mr. John').toString()));

xmlDoc = "<employee id='1'><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>"

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.replace('phone','542144')",
    "<employee id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>",
             (MYXML = new XML(xmlDoc), MYXML.replace('phone','542144').toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.replace('firstname','Johnny')",
    "<employee id=\"1\">Johnny<lastname>Walton</lastname><age>25</age></employee>",
             (MYXML = new XML(xmlDoc), MYXML.replace('firstname','Johnny').toString()));

var expectedResult;
expectedResult = '<phone>1234567</phone>';

// This should replace all the children
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.replace('*','<phone>1234567</phone>')",
    expectedResult,
             (MYXML = new XML(xmlDoc), MYXML.replace('*',"<phone>1234567</phone>").toString()));

// What about using an attribute name as a input parameter
// !!@ Rhino does an attribute addition after id!?!?
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.replace('@phone','<phone>7654321</phone>')",
    "<employee id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>",
             (MYXML = new XML(xmlDoc), MYXML.replace('@phone',"<phone>7654321</phone>").toString()));



END();
