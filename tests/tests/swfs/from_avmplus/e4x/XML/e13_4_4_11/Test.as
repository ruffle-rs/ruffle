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

START("13.4.4.11 - XML copy()");

//TEST(1, true, XML.prototype.hasOwnProperty("copy"));

emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

correct =
<employee id="0"><name>Jim</name><age>25</age></employee>;

x1 = emps.employee[0].copy();

TEST(2, undefined, x1.parent());
TEST(3, correct, x1);
 
// Make sure we're getting a copy, not a ref to orig.
emps =
<employees>
    <employee id="0"><fname>Jim</fname><age>25</age></employee>
    <employee id="1"><fname>Joe</fname><age>20</age></employee>
</employees>;

correct =
<employee id="0"><fname>Jim</fname><age>25</age></employee>

empCopy = emps.employee[0].copy();

emps.employee[0].fname[0] = "Sally";

TEST(4, correct, empCopy);

// Try copying whole XML twice
emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

empCopy = emps.copy();
x1 = empCopy.copy();

TEST(5, x1, emps);

XML.prettyPrinting = false;
var xmlDoc = "<MLB><Team>Giants</Team><City>San Francisco</City><Team>Padres</Team><City>San Diego</City></MLB>";

Assert.expectEq( "MYXML = new XML(xmlDoc), XMLCOPY = MYXML.copy()", xmlDoc,
             (MYXML = new XML(xmlDoc), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XML(null), XMLCOPY = MYXML.copy()", "",
             (MYXML = new XML(null), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XML(undefined), XMLCOPY = MYXML.copy()", MYXML.toString(),
             (MYXML = new XML(undefined), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XML(), XMLCOPY = MYXML.copy()", MYXML.toString(),
             (MYXML = new XML(), XMLCOPY = MYXML.copy(), XMLCOPY.toString()) );
Assert.expectEq( "MYXML = new XML(xmlDoc), XMLCOPY = MYXML.Team.copy()", "<Team>Giants</Team>" + NL() + "<Team>Padres</Team>",
             (MYXML = new XML(xmlDoc), XMLCOPY = MYXML.Team.copy(), XMLCOPY.toString()) );

// Make sure it's a copy
var MYXML = new XML(xmlDoc);
var MYXML2 = MYXML.copy();
Assert.expectEq ("MYXML == MYXML.copy()", true, (MYXML == MYXML.copy()));
MYXML2.foo = "bar";
Assert.expectEq ("MYXML == MYXML2 where MYXML2 is a copy that has been changed", false, (MYXML == MYXML2));

END();
