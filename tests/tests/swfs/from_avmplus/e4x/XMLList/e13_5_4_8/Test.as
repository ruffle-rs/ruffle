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

START("13.5.4.8 - XMLList copy()");

//TEST(1, true, XMLList.prototype.hasOwnProperty("copy"));

emps = new XMLList();
emps += <employee id="0"><name>Jim</name><age>25</age></employee>;
emps += <employee id="1"><name>Joe</name><age>20</age></employee>;

correct = new XMLList();
correct += <employee id="0"><name>Jim</name><age>25</age></employee>;
correct += <employee id="1"><name>Joe</name><age>20</age></employee>;

TEST(2, emps, emps.copy());
TEST(3, correct, emps.copy());

// Make sure we're getting a copy, not a ref to orig.
emps = new XMLList();
emps += <employee id="0"><name>Jim</name><age>25</age></employee>;
emps += <employee id="1"><name>Joe</name><age>20</age></employee>;
   
correct = new XMLList();
correct += <employee id="0"><name>Jim</name><age>25</age></employee>;
correct += <employee id="1"><name>Joe</name><age>20</age></employee>;

x1 = emps.copy();

emps += <employee id="2"><name>Sue</name><age>32</age></employee>;

TEST(4, correct, x1);

XML.prettyPrinting = false;
var xmlDoc = "<MLB><Team>Giants</Team><City>San Francisco</City><Team>Padres</Team><City>San Diego</City></MLB>";

Assert.expectEq( "MYXML = new XMLList(xmlDoc), XMLCOPY = MYXML.copy()", xmlDoc,
             (MYXML = new XMLList(xmlDoc), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XMLList(null), XMLCOPY = MYXML.copy()", "",
             (MYXML = new XMLList(null), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XMLList(undefined), XMLCOPY = MYXML.copy()", MYXML.toString(),
             (MYXML = new XMLList(undefined), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XMLList(), XMLCOPY = MYXML.copy()", MYXML.toString(),
             (MYXML = new XMLList(), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XMLList(xmlDoc), XMLCOPY = MYXML.Team.copy()", "<Team>Giants</Team>" + NL() + "<Team>Padres</Team>",
             (MYXML = new XMLList(xmlDoc), XMLCOPY = MYXML.Team.copy(), XMLCOPY.toString()) );

// Make sure it's a copy
var MYXML = new XMLList(xmlDoc);
var MYXML2 = MYXML.copy();
Assert.expectEq ("MYXML == MYXML.copy()", true, (MYXML == MYXML.copy()));
MYXML2.foo = "bar";
//MYXML2 = new XML("<blah>hi</blah>");
Assert.expectEq ("MYXML == MYXML2 where MYXML2 is a copy that has been changed", false, (MYXML == MYXML2));

END();
