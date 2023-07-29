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
import com.adobe.test.Utils;

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


START("13.4.4.3 - XML appendChild()");

//TEST(1, true, XML.prototype.hasOwnProperty("appendChild"));

// Add new employee to list
emps =
  <employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
  </employees>;

correct =
  <employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
    <employee id="2"><name>Sue</name><age>30</age></employee>
  </employees>;

newEmp = <employee id="2"><name>Sue</name><age>30</age></employee>;

emps.appendChild(newEmp);
TEST(2, correct, emps);

// Add a new child element to the end of Jim's employee element
emps =
  <employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
  </employees>;

correct =
  <employees>
    <employee id="0"><name>Jim</name><age>25</age><hobby>snorkeling</hobby></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
  </employees>;

emps.employee.(name == "Jim").appendChild(<hobby>snorkeling</hobby>);
TEST(3, correct, emps);

XML.prettyPrinting = false;
var xmlDoc = "<XML><TEAM>Giants</TEAM><TEAM>Padres</TEAM></XML>";

var expectedResult;
expectedResult = '<XML><TEAM>Giants</TEAM><TEAM>Padres</TEAM><TEAM>&lt;TEAM&gt;Red Sox&lt;/TEAM&gt;</TEAM></XML>';

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.appendChild('<TEAM>Red Sox</TEAM>'), MYXML.toXMLString()",
  expectedResult,
  (MYXML = new XML(xmlDoc), MYXML.appendChild('<TEAM>Red Sox</TEAM>'), MYXML.toXMLString()) );

expectedResult = '<XML><TEAM>Giants&lt;City&gt;San Francisco&lt;/City&gt;</TEAM><TEAM>Padres</TEAM></XML>';
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.TEAM[0].appendChild ('<City>San Francisco</City>')), MYXML.toXMLString()",
  expectedResult,
  (MYXML = new XML(xmlDoc), MYXML.TEAM[0].appendChild ('<City>San Francisco</City>'), MYXML.toXMLString()) );


// Weird behavior of new XML(null) vs XML()

var child = new XML("<TEAM>Giants</TEAM>");
var xml = new XML("foo");

Assert.expectEq( "MYXML = new XML(null), MYXML.appendChild(new XML('<TEAM>Giants</TEAM>')), MYXML.nodeKind()",
  "text",
  (MYXML = new XML(null), MYXML.appendChild(new XML("<TEAM>Giants</TEAM>")), MYXML.nodeKind()) );

Assert.expectEq( "MYXML = new XML(null), MYXML.appendChild(new XML('<TEAM>Giants</TEAM>')), MYXML.toString()",
  "",
  (MYXML = new XML(null), MYXML.appendChild(new XML("<TEAM>Giants</TEAM>")), MYXML.toString()) );

// This has weird behavior of seemingly replacing the XML() node with the appended child.  It somehow
// converts the "text" (really "empty") node into a element node
// 03/07/05 [vfleisch] NOT ANYMORE. Updated test case to assert for text, instead of element.
Assert.expectEq( "MYXML = new XML(), MYXML.appendChild(new XML('<TEAM>Giants</TEAM>')), MYXML.nodeKind()",
  "text",
  (MYXML = new XML(), MYXML.appendChild(new XML("<TEAM>Giants</TEAM>")), MYXML.nodeKind()) );

MYXML = new XML();
MYXML.appendChild(new XML("<TEAM>Giants</TEAM>"));

var MYXML = new XML('<LEAGUE></LEAGUE>');
var x1 = new XML('<TEAM>Giants</TEAM>');
MYXML.appendChild(x1);

Assert.expectEq( "move child node - MYXML.appendChild(new XML('<TEAM>Giants</TEAM>')), MYXML.toString()",
  "<LEAGUE><TEAM>Giants</TEAM></LEAGUE>",
  (MYXML.appendChild(x1), MYXML.toString()) );

MYXML = new XML('<LEAGUE></LEAGUE>');
x1 = new XML('<TEAM>Giants</TEAM>');
MYXML.appendChild(x1);

Assert.expectEq( "true move child node - MYXML.appendChild(MYXML.child(0)[0]), MYXML.toString()",
  "<LEAGUE><TEAM>Giants</TEAM></LEAGUE>",
  (MYXML.appendChild(MYXML.child(0)[0]), MYXML.toString()) );

expectedResult = '<b>a</b>';

MYXML = new XML('<?xml version="1.0"?><root></root>');
Assert.expectEq( "MYXML = new XML('<?xml version=\"1.0\"?><root></root>'); MYXML.appendChild(\"<b>a</b>\"), MYXML.toString()",
  expectedResult,
  (MYXML.appendChild("<b>a</b>"), MYXML.toString()));

MYXML = new XML('<LEAGUE></LEAGUE>');
x1 = new XMLList('<TEAM t="a">Giants</TEAM><TEAM t="b">Robots</TEAM>');
MYXML.appendChild(x1);

Assert.expectEq( "Append XMLList",
  '<LEAGUE><TEAM t="a">Giants</TEAM><TEAM t="b">Robots</TEAM></LEAGUE>',
  (MYXML.toString()) );

MYXML = new XML('<SCARY><MOVIE></MOVIE></SCARY>');
x1 = "poltergeist";
MYXML.MOVIE.appendChild(x1);

Assert.expectEq( "Append a string to child node",
  '<SCARY><MOVIE>poltergeist</MOVIE></SCARY>',
  (MYXML.toString()) );


MYXML = new XML('<SCARY><MOVIE></MOVIE></SCARY>');
x1 = "poltergeist";
MYXML.appendChild(x1);

expectedResult = '<SCARY><MOVIE/><MOVIE>poltergeist</MOVIE></SCARY>';

Assert.expectEq( "Append a string to top node",
  expectedResult,
  (MYXML.toString()) );

MYXML = new XML('<SCARY><MOVIE></MOVIE></SCARY>');
x1 = new XML("<the>poltergeist</the>");
MYXML.appendChild(x1);

Assert.expectEq( "Append a node to child node",
  '<SCARY><MOVIE/><the>poltergeist</the></SCARY>',
  (MYXML.toString()) );

var a = <a><b><c/></b></a>;

try {
  a.appendChild (a);
  result = a;
} catch (e1) {
  result = Utils.typeError(e1.toString());
}

Assert.expectEq("a = <a><b><c/></b></a>, a.appendChild(a)", "TypeError: Error #1118", result);


END();
