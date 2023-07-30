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

START("13.4.4.4 - XML attribute()");

//TEST(1, true, XML.prototype.hasOwnProperty("attribute"));

// Get count of employees
emps =
<employees count="2">
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

TEST_XML(2, 2, emps.attribute("count"));

// Get the id of the employee age 25
emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

TEST_XML(3, 0, emps.employee.(age == "25").attribute("id"));

// Get the id of the employee named Jim
emps =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

TEST_XML(4, 0, emps.employee.(name == "Jim").attribute("id"));

var xmlDoc = "<TEAM foo = 'bar' two='second'>Giants</TEAM>";

// verify that attribute correct finds one attribute node
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('foo') instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.attribute('foo') instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('foo') instanceof XML", false,
             (MYXML = new XML(xmlDoc), MYXML.attribute('foo') instanceof XML ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('foo').length()", 1,
             (MYXML = new XML(xmlDoc), MYXML.attribute('foo').length() ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('foo').toString()", "bar",
             (MYXML = new XML(xmlDoc), MYXML.attribute('foo').toString() ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('foo')[0].nodeKind()", "attribute",
             (MYXML = new XML(xmlDoc), MYXML.attribute('foo')[0].nodeKind() ));

// verify that attribute doesn't find non-existent names
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XML", false,
             (MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST') instanceof XML ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST').length()", 0,
             (MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST').length() ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST').toString()", "",
             (MYXML = new XML(xmlDoc), MYXML.attribute('DOESNOTEXIST').toString() ));
             
// verify that attribute doesn't find child node names
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('TEAM') instanceof XMLList", true,
             (MYXML = new XML(xmlDoc), MYXML.attribute('TEAM') instanceof XMLList ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('TEAM') instanceof XML", false,
             (MYXML = new XML(xmlDoc), MYXML.attribute('TEAM') instanceof XML ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('TEAM').toString()", "",
             (MYXML = new XML(xmlDoc), MYXML.attribute('TEAM').toString() ));
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.attribute('TEAM').length()", 0,
             (MYXML = new XML(xmlDoc), MYXML.attribute('TEAM').length() ));

xl = <x a="aatr" b="batr">y</x>;

Assert.expectEq( "attribute(new QName(\"*\"))", "aatrbatr",
       ( q = new QName("*"), xl.attribute(q).toString() ));

Assert.expectEq( "attribute(new QName(\"@*\"))", "",
       ( q = new QName("@*"), xl.attribute(q).toString() ));

END();
