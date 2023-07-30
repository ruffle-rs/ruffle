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

START("13.4.4.10 - XML contains()");

//TEST(1, true, XML.prototype.hasOwnProperty("contains"));

emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

TEST(2, true, emps.contains(emps));

var xmlDoc = "<MLB><Team>Giants</Team><City>San Francisco</City><Team>Padres</Team><City>San Diego</City></MLB>";

// same object, returns true
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.contains(MYMXL)", true,
             (MYXML = new XML(xmlDoc), MYXML.contains(MYXML)) );
             
// duplicated object, returns true
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.contains(MYMXL.copy())", true,
             (MYXML = new XML(xmlDoc), MYXML.contains(MYXML.copy())) );
             
// identical objects, returns true
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML.contains(MYMXL2)", true,
             (MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML.contains(MYXML2)) );
             
// identical objects, returns true
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML2.contains(MYMXL)", true,
             (MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML2.contains(MYXML)) );
             
// slightly different objects, returns false
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML2.foo = 'bar', MYXML.contains(MYMXL2)", false,
             (MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML2.foo = 'bar', MYXML.contains(MYXML2)) );

// slightly different objects #2, returns false
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML2.Team[0].foo = 'bar', MYXML.contains(MYMXL2)", false,
             (MYXML = new XML(xmlDoc), MYXML2 = new XML(xmlDoc), MYXML2.Team[0].foo = 'bar', MYXML.contains(MYXML2)) );

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.Team[0].contains('Giants')", false,
             (MYXML = new XML(xmlDoc), MYXML.Team[0].contains('Giants')) );
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.Team[1].contains('Giants')", false,
             (MYXML = new XML(xmlDoc), MYXML.Team[1].contains('Giants')) );
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.City.contains('Giants')", false,
             (MYXML = new XML(xmlDoc), MYXML.City.contains('Giants')) );

END();
