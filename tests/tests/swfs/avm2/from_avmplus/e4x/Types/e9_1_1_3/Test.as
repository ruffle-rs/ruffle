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

START("9.1.1.3 - XML [[Delete]]");

// .@
x1 =
<alpha attr1="value1">one</alpha>;

delete x1.@attr1;
TEST_XML(1, "", x1.@attr1);
TEST(2, <alpha>one</alpha>, x1);

// ..@
x1 =
<alpha attr1="value1">
    one
    <bravo attr2="value2">
        two
        <charlie attr3="value3">three</charlie>
    </bravo>
</alpha>;

xref =
<alpha attr1="value1">
    one
    <bravo attr2="value2">
        two
        <charlie attr3="value3">three</charlie>
    </bravo>
</alpha>;

correct =
<alpha attr1="value1">
    one
    <bravo attr2="value2">
        two
        <charlie>three</charlie>
    </bravo>
</alpha>;

delete x1..@attr3;

//Rhino test:
//TEST(3, correct, x1);

// We are different. Deleting descendant attributes doesn't change the original object:
TEST(3, xref.toString(), x1.toString());


// ..@*
x1 =
<alpha attr1="value1">
    one
    <bravo attr2="value2">
        two
        <charlie attr3="value3">three</charlie>
    </bravo>
</alpha>;

xref =
<alpha attr1="value1">
    one
    <bravo attr2="value2">
        two
        <charlie attr3="value3">three</charlie>
    </bravo>
</alpha>;

correct =
<alpha>
    one
    <bravo>
        two
        <charlie>three</charlie>
    </bravo>
</alpha>;

delete x1..@*;

// Rhino test:
//TEST(4, correct, x1);

// We are different. Deleting descendant attributes doesn't change the original object:
TEST(4, xref.toString(), x1.toString());

END();
