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

START("10.4.1 - toXMLList Applied to String type");

var x1, y1, correct;

x1 =
<>
    <alpha>one</alpha>
    <bravo>two</bravo>
</>;

TEST(1, "xml", typeof(x1));
TEST(2, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());

// Load from another XMLList
// Make sure it is copied if it's an XMLList
y1 = new XMLList(x1);

x1 += <charlie>three</charlie>;

TEST(3, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", y1.toXMLString());
   
// Load from one XML type
x1 = new XMLList(<alpha>one</alpha>);
TEST(4, "<alpha>one</alpha>", x1.toXMLString());

// Load from Anonymous
x1 = new XMLList(<><alpha>one</alpha><bravo>two</bravo></>);
TEST(5, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());

// Load from Anonymous as string
x1 = new XMLList("<alpha>one</alpha><bravo>two</bravo>");
TEST(6, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());

// Load from illegal type
//x1 = new XMLList("foobar");
//ADD(7, "", x);

John = "<employee><name>John</name><age>25</age></employee>";
Sue = "<employee><name>Sue</name><age>32</age></employee>";

correct =
<>
    <employee><name>John</name><age>25</age></employee>
    <employee><name>Sue</name><age>32</age></employee>
</>;

var1 = new XMLList(John + Sue);

TEST(8, correct, var1);

END();
