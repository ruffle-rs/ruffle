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

START("9.2.1.1 XMLList [[Get]]");

var x1 =
<>
<alpha attr1="value1">
    <bravo attr2="value2">
        one
        <charlie>two</charlie>
    </bravo>
</alpha>
<alpha attr1="value3">
    <bravo attr2="value4">
        three
        <charlie>four</charlie>
    </bravo>
</alpha>
</>;

// .
correct =
<>
    <bravo attr2="value2">
        one
        <charlie>two</charlie>
    </bravo>
    <bravo attr2="value4">
        three
        <charlie>four</charlie>
    </bravo>
</>;

TEST(1, correct, x1.bravo);

correct =
<>
    <charlie>two</charlie>
    <charlie>four</charlie>
</>;

TEST(2, correct, x1.bravo.charlie);

// .@
correct = new XMLList();
correct += new XML("value1");
correct += new XML("value3");
TEST(3, correct, x1.@attr1);

correct = new XMLList();
correct += new XML("value2");
correct += new XML("value4");
TEST(4, correct, x1.bravo.@attr2);

// ..
correct =
<>
    <bravo attr2="value2">
        one
        <charlie>two</charlie>
    </bravo>
    <bravo attr2="value4">
        three
        <charlie>four</charlie>
    </bravo>
</>;

TEST(5, correct, x1..bravo);

correct =
<>
    <charlie>two</charlie>
    <charlie>four</charlie>
</>;

TEST(6, correct, x1..charlie);

// .@*
correct = new XMLList();
correct += new XML("value1");
correct += new XML("value3");
TEST(7, correct, x1.@*);

x1 =
<alpha attr1="value1" attr2="value2">
    <bravo>
        one
        <charlie>two</charlie>
    </bravo>
</alpha>;

// ..*
correct = <><bravo>one<charlie>two</charlie></bravo>one<charlie>two</charlie>two</>;

XML.prettyPrinting = false;
TEST(8, correct, x1..*);
XML.prettyPrinting = true;

x1 =
<alpha attr1="value1" attr2="value2">
    <bravo attr2="value3">
        one
        <charlie attr3="value4">two</charlie>
    </bravo>
</alpha>;

// ..@
correct = new XMLList();
correct += new XML("value2");
correct += new XML("value3");
TEST(9, correct, x1..@attr2)

// ..@*
correct = new XMLList();
correct += new XML("value1");
correct += new XML("value2");
correct += new XML("value3");
correct += new XML("value4");
TEST(10, correct, x1..@*);


END();
