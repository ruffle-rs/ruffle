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

START("13.4.4.8 - XML children()");

//TEST(1, true, XML.prototype.hasOwnProperty("children"));

emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

correct = new XMLList();
correct += <employee id="0"><name>Jim</name><age>25</age></employee>;
correct += <employee id="1"><name>Joe</name><age>20</age></employee>;

TEST(2, "xml", typeof(emps.children()));
TEST(3, correct, emps.children());

// Get the child elements of the first employee
correct = new XMLList();
correct += <name>Jim</name>,
correct += <age>25</age>;

TEST(4, correct, emps.employee[0].children());

var xmlDoc = "<company><employee id='1'><name>John</name> <city>California</city> </employee> <employee id='2'><name>Mary</name><city>Texas</city></employee></company>";

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[0].children().toString()", "<name>John</name>" + NL() + "<city>California</city>", (MYXML = new XML(xmlDoc), MYXML.employee[0].children().toString()));

// Same results whether or not prettyPrinting is true (XMLList.toString testing)
XML.prettyPrinting = false;
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[0].children().toString()", "<name>John</name>" + NL() + "<city>California</city>", (MYXML = new XML(xmlDoc), MYXML.employee[0].children().toString()));

//!!@This crashes ASC (because of the (id == '1') code
//!!@This does not work in Rhino
//!!@Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee.(id == '1').children()", "<name>John</name>, <city>California</city>", (MYXML = new XML(xmlDoc), MYXML.employee.(id == '1').children()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[1].children().toString()", "<name>Mary</name>" + NL() + "<city>Texas</city>", (MYXML = new XML(xmlDoc), MYXML.employee[1].children().toString()));

END();
