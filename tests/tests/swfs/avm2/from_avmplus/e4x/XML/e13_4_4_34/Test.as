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
 

START("13.4.4.34 - XML setLocalName()");

//TEST(1, true, XML.prototype.hasOwnProperty("setLocalName"));

x1 =
<alpha>
    <bravo>one</bravo>
</alpha>;

correct =
<charlie>
    <bravo>one</bravo>
</charlie>;

x1.setLocalName("charlie");

TEST(2, correct, x1);

x1 =
<ns:alpha xmlns:ns="http://foobar/">
    <ns:bravo>one</ns:bravo>
</ns:alpha>;

correct =
<ns:charlie xmlns:ns="http://foobar/">
    <ns:bravo>one</ns:bravo>
</ns:charlie>;

x1.setLocalName("charlie");

TEST(3, correct, x1);

XML.prettyPrinting = false;
var xmlDoc = "<employee id='1'><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee>"

// propertyName as a string
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setLocalName('newlocalname'),MYXML.localName()",
    "newlocalname",
    (MYXML = new XML(xmlDoc), MYXML.setLocalName('newlocalname'),MYXML.localName()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setLocalName('newlocalname'),MYXML.toString()",
    "<newlocalname id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></newlocalname>",
    (MYXML = new XML(xmlDoc), MYXML.setLocalName('newlocalname'),MYXML.toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setLocalName(new QName('newlocalname')),MYXML.toString()",
    "<newlocalname id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></newlocalname>",
    (MYXML = new XML(xmlDoc), MYXML.setLocalName(QName('newlocalname')),MYXML.toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setLocalName(new QName('foo', 'newlocalname')),MYXML.toString()",
    "<newlocalname id=\"1\"><firstname>John</firstname><lastname>Walton</lastname><age>25</age></newlocalname>",
    (MYXML = new XML(xmlDoc), MYXML.setLocalName(QName('foo', 'newlocalname')),MYXML.toString()));

MYXML = new XML(xmlDoc);

try {
    MYXML.setLocalName('@newlocalname');
    result = "no error";
} catch (e1) {
    result = Utils.typeError(e1.toString());
}

Assert.expectEq( "setLocalName('@newlocalname')", "TypeError: Error #1117", result);


try {
    MYXML.setLocalName('*');
    result = "no error";
} catch (e2) {
    result = Utils.typeError(e2.toString());
}

Assert.expectEq( "setLocalName('*')", "TypeError: Error #1117", result);

try {
    MYXML.setLocalName('x123=5');
    result = "no error";
} catch (e3) {
    result = Utils.typeError(e3.toString());
}

Assert.expectEq( "setLocalName('x123=5')", "TypeError: Error #1117", result);

try {
    MYXML.setLocalName('123');
    result = "no error";
} catch (e4) {
    result = Utils.typeError(e4.toString());
}

Assert.expectEq( "setLocalName('123')", "TypeError: Error #1117", result);

try {
    MYXML.setLocalName('!bam');
    result = "no error";
} catch (e5) {
    result = Utils.typeError(e5.toString());
}

Assert.expectEq( "setLocalName('!bam')", "TypeError: Error #1117", result);

END();
