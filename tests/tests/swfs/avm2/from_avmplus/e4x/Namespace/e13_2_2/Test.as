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

START("13.2.2 - Namespace Constructor");

n = new Namespace();
TEST(1, "object", typeof(n));
TEST(2, "", n.prefix);
TEST(3, "", n.uri);

n = new Namespace("");
TEST(4, "object", typeof(n));
TEST(5, "", n.prefix);
TEST(6, "", n.uri);

n = new Namespace("http://foobar/");
TEST(7, "object", typeof(n));
TEST(8, "undefined", typeof(n.prefix));
TEST(9, "http://foobar/", n.uri);

// Check if the undefined prefix is getting set properly
m = new Namespace(n);
TEST(10, typeof(n), typeof(m));
TEST(11, n.prefix, m.prefix);
TEST(12, n.uri, m.uri);

n = new Namespace("foobar", "http://foobar/");
TEST(13, "object", typeof(n));
TEST(14, "foobar", n.prefix);
TEST(15, "http://foobar/", n.uri);

// Check if all the properties are getting copied
m = new Namespace(n);
TEST(16, typeof(n), typeof(m));
TEST(17, n.prefix, m.prefix);
TEST(18, n.uri, m.uri);

try {
    n = new Namespace("ns", "");
    SHOULD_THROW(19);
} catch(ex) {
    TEST(19, "TypeError", ex.name);
}

namespace foo = "bar";

Assert.expectEq("Access inline namespace by name", "bar", foo.toString());

x1 = new Namespace ("p", "y");

Assert.expectEq("Access instantiated namespace by name", "y", x1.toString());

END();
