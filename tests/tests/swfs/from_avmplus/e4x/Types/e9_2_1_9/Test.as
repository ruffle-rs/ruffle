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

START("9.2.1.9 XMLList [[Equals]]");

// Empty list should equal undefined
TEST(1, true, (new XMLList() == undefined) && (undefined == new XMLList()));

// Compare two lists if all are equal
x1 = <alpha>one</alpha> + <bravo>two</bravo>;
y1 = <alpha>one</alpha> + <bravo>two</bravo>;
TEST(2, true, (x1 == y1) && (y1 == x1));
y1 = <alpha>one</alpha> + <bravo>two</bravo> + <charlie>three</charlie>;
TEST(3, false, (x1 == y1) || (y1 == x1));
y1 = <alpha>one</alpha> + <bravo>not</bravo>;
TEST(4, false, (x1 == y1) || (y1 == x1));

// If XMLList has one argument should compare with just the 0th element.
x1= new XMLList("<alpha>one</alpha>");
y1 = <alpha>one</alpha>;
TEST(5, true, (x1 == y1) && (y1 == x1));
y1 = "one";
TEST(6, true, (x1 == y1) && (y1 == x1));

// Should return false even if list contains element equal to comparison
x1 = <alpha>one</alpha> + <bravo>two</bravo>;
y1 = <alpha>one</alpha>;
TEST(7, false, (x1 == y1) && (y1 == x1));

y1 = "<alpha>one</alpha>";
TEST(8, false, (x1 == y1) || (y1 == x1));

// Try other types - should return false
y1 = null;
TEST(9, false, (x1 == y1) || (y1 == x1));

y1 = new Object();
TEST(10, false, (x1 == y1) || (y1 == x1));

END();
