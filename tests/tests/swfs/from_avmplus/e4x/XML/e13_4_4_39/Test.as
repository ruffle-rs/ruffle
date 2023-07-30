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

START("13.4.4.39 - XML toXMLString");

//TEST(1, true, XML.prototype.hasOwnProperty("toXMLString"));

XML.prettyPrinting = false;

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>
        <bravo>two</bravo>
    </charlie>
</alpha>;

TEST(2, "<bravo>one</bravo>", x1.bravo.toXMLString());
TEST(3, "<bravo>one</bravo>" + NL() + "<bravo>two</bravo>", x1..bravo.toXMLString());

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie/>
</alpha>;

TEST(4, "<charlie/>", x1.charlie.toXMLString());

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>
        <bravo>two</bravo>
    </charlie>
</alpha>;

TEST(5, "<charlie><bravo>two</bravo></charlie>", x1.charlie.toXMLString());

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>
        two
        <bravo/>
    </charlie>
</alpha>;

TEST(6, "<charlie>two<bravo/></charlie>", x1.charlie.toXMLString());

x1 =
<alpha>
    <bravo></bravo>
    <bravo/>
</alpha>;

TEST(7, "<bravo/>" + NL() + "<bravo/>", x1.bravo.toXMLString());

x1 =
<alpha>
    <bravo>one<charlie/></bravo>
    <bravo>two<charlie/></bravo>
</alpha>;

TEST(8, "<bravo>one<charlie/></bravo>" + NL() + "<bravo>two<charlie/></bravo>", x1.bravo.toXMLString());
   
XML.prettyPrinting = true;

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>two</charlie>
</alpha>;

copy = x1.bravo.copy();

TEST(9, "<bravo>one</bravo>", copy.toXMLString());

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>
        <bravo>two</bravo>
    </charlie>
</alpha>;

TEST(10, "String contains value one from bravo", "String contains value " + x1.bravo + " from bravo");

var xmlDoc = "<employees><employee id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee><employee id=\"2\"><firstname>Sue</firstname><lastname>Day</lastname><age>32</age></employee></employees>"


// propertyName as a string
XML.prettyPrinting = false;
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[0].toXMLString()",
    "<employee id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>",
    (MYXML = new XML(xmlDoc), MYXML.employee[0].toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[1].toXMLString()",
    "<employee id=\"2\"><firstname>Sue</firstname><lastname>Day</lastname><age>32</age></employee>",
    (MYXML = new XML(xmlDoc), MYXML.employee[1].toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[0].firstname.toXMLString()",
    "<firstname>John</firstname>",
    (MYXML = new XML(xmlDoc), MYXML.employee[0].firstname.toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[1].firstname.toXMLString()",
    "<firstname>Sue</firstname>",
    (MYXML = new XML(xmlDoc), MYXML.employee[1].firstname.toXMLString()));

// test case containing xml namespaces
var xmlDoc2 = "<ns2:table2 foo=\"5\" xmlns=\"http://www.w3schools.com/furniture\" ns2:bar=\"last\" xmlns:ns2=\"http://www.macromedia.com/home\"><ns2:name>African Coffee Table</ns2:name><width>80</width><length>120</length></ns2:table2>";
Assert.expectEq( "MYXML = new XML(xmlDoc2), MYXML.toXMLString()",
    "<ns2:table2 foo=\"5\" ns2:bar=\"last\" xmlns=\"http://www.w3schools.com/furniture\" xmlns:ns2=\"http://www.macromedia.com/home\"><ns2:name>African Coffee Table</ns2:name><width>80</width><length>120</length></ns2:table2>",
    (MYXML = new XML(xmlDoc2), MYXML.toXMLString()));


XML.prettyPrinting = true;
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[0].toXMLString()",
    "<employee id=\"1\">" + NL() + "  <firstname>John</firstname>" + NL() + "  <lastname>Walton</lastname>" + NL() + "  <age>25</age>" + NL() + "</employee>",
    (MYXML = new XML(xmlDoc), MYXML.employee[0].toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[1].toXMLString()",
    "<employee id=\"2\">" + NL() + "  <firstname>Sue</firstname>" + NL() + "  <lastname>Day</lastname>" + NL() + "  <age>32</age>" + NL() + "</employee>",
    (MYXML = new XML(xmlDoc), MYXML.employee[1].toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[0].firstname.toXMLString()",
    "<firstname>John</firstname>",
    (MYXML = new XML(xmlDoc), MYXML.employee[0].firstname.toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.employee[1].firstname.toXMLString()",
    "<firstname>Sue</firstname>",
    (MYXML = new XML(xmlDoc), MYXML.employee[1].firstname.toXMLString()));

xmlDoc = "<XML>foo</XML>";
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.toXMLString()",
    "<XML>foo</XML>",
    (MYXML = new XML(xmlDoc), MYXML.toXMLString()));
    
END();
