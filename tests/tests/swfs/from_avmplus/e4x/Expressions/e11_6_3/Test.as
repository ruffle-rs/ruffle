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

START("11.6.3 - Compound Assignment");

// Insert employee 3 and 4 after the first employee
e =
<employees>
    <employee id="1">
        <name>Joe</name>
        <age>20</age>
    </employee>
    <employee id="2">
        <name>Sue</name>
        <age>30</age>
    </employee>
</employees>;

correct =
<employees>
    <employee id="1">
        <name>Joe</name>
        <age>20</age>
    </employee>
    <employee id="3">
        <name>Fred</name>
    </employee>
    <employee id="4">
        <name>Carol</name>
    </employee>
    <employee id="2">
        <name>Sue</name>
        <age>30</age>
    </employee>
</employees>;
    
e.employee[0] += <employee id="3"><name>Fred</name></employee> +
    <employee id="4"><name>Carol</name></employee>;
    
TEST(1, correct, e);

// Append employees 3 and 4 to the end of the employee list
e =
<employees>
    <employee id="1">
        <name>Joe</name>
        <age>20</age>
    </employee>
    <employee id="2">
        <name>Sue</name>
        <age>30</age>
    </employee>
</employees>;

correct =
<employees>
    <employee id="1">
        <name>Joe</name>
        <age>20</age>
    </employee>
    <employee id="2">
        <name>Sue</name>
        <age>30</age>
    </employee>
    <employee id="3">
        <name>Fred</name>
    </employee>
    <employee id="4">
        <name>Carol</name>
    </employee>
</employees>;

e.employee[1] += <employee id="3"><name>Fred</name></employee> +
    <employee id="4"><name>Carol</name></employee>;
TEST(2, correct, e);
       
// XML +=

var x1 = new XML("<a><b><c>A0</c><d>A1</d></b><b><c>B0</c><d>B1</d></b><b><c>C0</c><d>C1</d></b></a>");

x1.b[1] += new XML("<b><c>D0</c><d>D1</d></b>");

var y1 = new XML("<a><b><c>A0</c><d>A1</d></b><b><c>B0</c><d>B1</d></b><b><c>D0</c><d>D1</d></b><b><c>C0</c><d>C1</d></b></a>");

Assert.expectEq( "XML +=     :", true, (x1==y1) );


// XMLList +=

x1 = new XMLList("<a><b>A0</b><c>A1</c></a><a><b>B0</b><c>B1</c></a><a><b>C0</b><c>C1</c></a>");

x1 += new XML("<a><b>D0</b><c>D1</c></a>");

y1 = new XMLList("<a><b>A0</b><c>A1</c></a><a><b>B0</b><c>B1</c></a><a><b>C0</b><c>C1</c></a><a><b>D0</b><c>D1</c></a>");

Assert.expectEq( "XMLList += :", true, (x1==y1) );


// XMLList +=, last item in XMLList is XML object with non-null parent

x1 = new XML("<a><b><c>A0</c><d>A1</d></b><b><c>B0</c><d>B1</d></b><b><c>C0</c><d>C1</d></b></a>");

x1 += new XMLList("<b><c>D0</c></b><b><c>E0</c></b>");

y1 = new XMLList("<a><b><c>A0</c><d>A1</d></b><b><c>B0</c><d>B1</d></b><b><c>C0</c><d>C1</d></b></a><b><c>D0</c></b><b><c>E0</c></b>");

Assert.expectEq( "XMLList += :", true, (x1==y1) );

END();
