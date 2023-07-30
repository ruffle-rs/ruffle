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

START("10.4 - toXMLList");

var xl;

// null
try {
    xl = null;
    xl.toString();
    SHOULD_THROW(1);
} catch (ex) {
    TEST(1, "TypeError", ex.name);
}

// number
x = new Number(123);
TEST(2, "123", new XMLList(x).toXMLString());

// String
x = new String("<alpha><bravo>one</bravo></alpha>");
TEST(3, <alpha><bravo>one</bravo></alpha>, new XMLList(x));

x = new String("<alpha>one</alpha><charlie>two</charlie>");
TEST(4, "<alpha>one</alpha>" + "\n" + "<charlie>two</charlie>",
  new XMLList(x).toXMLString());

// XML
x = new XML(<alpha><bravo>one</bravo></alpha>);
TEST(5, <alpha><bravo>one</bravo></alpha>, new XMLList(x));

x = new XML(<root><alpha><bravo>one</bravo></alpha><charlie>two</charlie></root>);
TEST(6, <alpha><bravo>one</bravo></alpha>, new XMLList(x.alpha));

// XMLList
x = new XMLList(<alpha><bravo>one</bravo></alpha>);
TEST(7, <alpha><bravo>one</bravo></alpha>, new XMLList(x));

x = new XMLList(<><alpha>one</alpha><bravo>two</bravo></>);
TEST(8, "<alpha>one</alpha>" + "\n" + "<bravo>two</bravo>",
  new XMLList(x).toXMLString());
  
END();
