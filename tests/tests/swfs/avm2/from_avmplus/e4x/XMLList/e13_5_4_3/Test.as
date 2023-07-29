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

START("13.5.4.3 - XMLList attributes()");

//TEST(1, true, XMLList.prototype.hasOwnProperty("attributes"));

// Test with XMLList of size 0
x1 = new XMLList()
TEST(2, "xml", typeof(x1.attributes()));
TEST_XML(3, "", x1.attributes());

// Test with XMLList of size 1
x1 += <alpha attr1="value1" attr2="value2">one</alpha>;

TEST(4, "xml", typeof(x1.attributes()));
correct = new XMLList();
correct += new XML("value1");
correct += new XML("value2");
TEST(5, correct, x1.attributes());

// Test with XMLList of size > 1
x1 += <bravo attr3="value3" attr4="value4">two</bravo>;

TEST(6, "xml", typeof(x1.attributes()));
correct = new XMLList();
correct += new XML("value1");
correct += new XML("value2");
correct += new XML("value3");
correct += new XML("value4");
TEST(7, correct, x1.attributes());

var xmlDoc = "<TEAM foo = 'bar' two='second'>Giants</TEAM><CITY two = 'third'>Giants</CITY>";

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes() instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.attributes() instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes() instanceof XML", false,
             (MYXML = new XMLList(xmlDoc), MYXML.attributes() instanceof XML ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes().length()", 3,
             (MYXML = new XMLList(xmlDoc), MYXML.attributes().length() ));
XML.prettyPrinting = false;
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes().toString()", "barsecondthird",
             (MYXML = new XMLList(xmlDoc), MYXML.attributes().toString() ));
XML.prettyPrinting = true;
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes().toString()", "barsecondthird",
             (MYXML = new XMLList(xmlDoc), MYXML.attributes().toString() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes()[0].nodeKind()", "attribute",
             (MYXML = new XMLList(xmlDoc), MYXML.attributes()[0].nodeKind() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes()[1].nodeKind()", "attribute",
             (MYXML = new XMLList(xmlDoc), MYXML.attributes()[1].nodeKind() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes()[2].nodeKind()", "attribute",
             (MYXML = new XMLList(xmlDoc), MYXML.attributes()[2].nodeKind() ));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.attributes().toXMLString()", "bar" + NL() + "second" + NL() + "third",
             (MYXML = new XMLList(xmlDoc), MYXML.attributes().toXMLString() ));

END();
