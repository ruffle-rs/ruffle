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

START("13.5.4.20 - XMLList toString()");

//TEST(1, true, XMLList.prototype.hasOwnProperty("toString"));
    
x1 = <><alpha>one</alpha></>;

TEST(2, "one", x1.toString());

x1 = <><alpha>one</alpha><bravo>two</bravo></>;

TEST(3, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", x1.toString());


var xmlDoc = "<employees><employee id='1'><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee><employee id='2'><firstname>Sue</firstname><lastname>Day</lastname><age>32</age></employee></employees>"


XML.prettyPrinting = false;
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[0].toString()",
    "<employee id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[0].toString()));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[1].toString()",
    "<employee id=\"2\"><firstname>Sue</firstname><lastname>Day</lastname><age>32</age></employee>",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[1].toString()));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[0].firstname.toString()",
    "John",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[0].firstname.toString()));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[1].firstname.toString()",
    "Sue",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[1].firstname.toString()));

XML.prettyPrinting = true;
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[0].toString()",
    "<employee id=\"1\">" + NL() + "  <firstname>John</firstname>" + NL() + "  <lastname>Walton</lastname>" + NL() + "  <age>25</age>" + NL() + "</employee>",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[0].toString()));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[1].toString()",
    "<employee id=\"2\">" + NL() + "  <firstname>Sue</firstname>" + NL() + "  <lastname>Day</lastname>" + NL() + "  <age>32</age>" + NL() + "</employee>",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[1].toString()));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[0].firstname.toString()",
    "John",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[0].firstname.toString()));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.employee[1].firstname.toString()",
    "Sue",
    (MYXML = new XMLList(xmlDoc), MYXML.employee[1].firstname.toString()));

xmlDoc = new XMLList("<XML>foo</XML>");

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.toString()",
    "foo",
    (MYXML = new XMLList(xmlDoc), MYXML.toString()));

END();
