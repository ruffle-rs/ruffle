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

START("9.1.1.9 - XML [[Equals]]");

x1 = <alpha>one</alpha>;
y1 = <alpha>one</alpha>;
TEST(1, true, (x1 == y1) && (y1 == x1));

// Should return false if comparison is not XML
y1 = "<alpha>one</alpha>";
TEST(2, false, (x1 == y1) || (y1 == x1));

y1 = undefined
TEST(3, false, (x1 == y1) || (y1 == x1));

y1 = null
TEST(4, false, (x1 == y1) || (y1 == x1));

y1 = new Object();
TEST(5, false, (x1 == y1) || (y1 == x1));

// Check with attributes
x1 = <alpha attr1="value1">one<bravo attr2="value2">two</bravo></alpha>;
y1 = <alpha attr1="value1">one<bravo attr2="value2">two</bravo></alpha>;
TEST(6, true, (x1 == y1) && (y1 == x1));

y1 = <alpha attr1="new value">one<bravo attr2="value2">two</bravo></alpha>;
TEST(7, false, (x1 == y1) || (y1 == x1));

// Logical equality
// Attribute order.
x1 = <alpha attr1="value1" attr2="value2">one<bravo attr3="value3" attr4="value4">two</bravo></alpha>;
y1 = <alpha attr2="value2" attr1="value1">one<bravo attr4="value4" attr3="value3">two</bravo></alpha>;
TEST(8, true, (x1 == y1) && (y1 == x1));

// Skips empty text nodes
x1 = <alpha> <bravo>one</bravo> </alpha>;
y1 = <alpha><bravo>one</bravo></alpha>;
TEST(9, true, (x1 == y1) && (y1 == x1));


XML.ignoreWhitespace = false;

// Doesn't trim text nodes.
x1 = <alpha><bravo> one </bravo></alpha>;
y1 = <alpha><bravo>one</bravo></alpha>;
TEST(10, false, (x1 == y1) || (y1 == x1));

// Compare comments
XML.ignoreComments = false;
x1 = new XML('<alpha><!-- comment1 --><bravo><!-- comment2 -->one</bravo></alpha>');
y1 = new XML('<alpha><!-- comment2 --><bravo><!-- comment1 -->one</bravo></alpha>');
TEST(11, false, (x1 == y1) || (y1 == x1));

one = x1.*[0];
two = y1.*[0];
TEST(12, false, (one == two) || (two == one));

one = x1.*[0];
two = y1.bravo.*[0];
TEST(13, true, (one == two) && (two == one));

 
// Compare processing instructions
XML.ignoreProcessingInstructions = false;
x1 = new XML('<alpha><?one foo="bar" ?><bravo><?two bar="foo" ?>one</bravo></alpha>');
y1 = new XML('<alpha><?two bar="foo" ?><bravo><?one foo="bar" ?>one</bravo></alpha>');
TEST(14, false, (x1 == y1) || (y1 == x1));

one = x1.*[0];
two = y1.*[0];
TEST(15, false, (one == two) || (two == one));

one = x1.*[0];
two = y1.bravo.*[0];
TEST(16, true, (one == two) && (two == one));

// Namespaces
x1 = <ns1:alpha xmlns:ns1="http://foo/"><ns1:bravo>one</ns1:bravo></ns1:alpha>;
y1 = <ns2:alpha xmlns:ns2="http://foo/"><ns2:bravo>one</ns2:bravo></ns2:alpha>;
 
TEST(17, true, (x1 == y1) && (y1 == x1));

y1 = <ns2:alpha xmlns:ns2="http://foo"><ns2:bravo>one</ns2:bravo></ns2:alpha>;
TEST(18, false, (x1 == y1) || (y1 == x1));


// Default namespace
default xml namespace = "http://foo/";
x1 = <alpha xmlns="http://foo/">one</alpha>;
y1 = <alpha>one</alpha>;

TEST(19, true, (x1 == y1) && (y1 == x1));


END();
