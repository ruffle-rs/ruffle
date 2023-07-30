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
 

START("13.4.4.19 - insertChildBefore()");

//TEST(1, true, XML.prototype.hasOwnProperty("insertChildBefore"));
    
x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>two</charlie>
</alpha>;

correct =
<alpha>
    <delta>three</delta>
    <bravo>one</bravo>
    <charlie>two</charlie>
</alpha>;

x1.insertChildBefore(x1.bravo[0], <delta>three</delta>);

TEST(2, correct, x1);

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>two</charlie>
</alpha>;

correct =
<alpha>
    <bravo>one</bravo>
    <charlie>two</charlie>
    <delta>three</delta>
</alpha>;
x2 = new XML();

x2 = x1.insertChildBefore(null, <delta>three</delta>);

TEST(3, correct, x1);

TEST(4, correct, x2);

// to simplify string matching
XML.prettyPrinting = false;

var xmlDoc = "<company></company>";
var child1 = new XML("<employee id='1'><name>John</name></employee>");
var child2 = new XML("<employee id='2'><name>Sue</name></employee>");
var child3 = new XML("<employee id='3'><name>Bob</name></employee>");

var allChildren = new XMLList("<employee id='1'><name>John</name></employee><employee id='2'><name>Sue</name></employee><employee id='3'><name>Bob</name></employee>");
var twoChildren = new XMLList("<employee id='1'><name>John</name></employee><employee id='2'><name>Sue</name></employee>");

var wholeString = "<company><employee id='1'><name>John</name></employee><employee id='2'><name>Sue</name></employee><employee id='3'><name>Bob</name></employee></company>";

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.toString()",
    "<company><employee id=\"1\"><name>John</name></employee></company>",
    (MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.children()[0].parent() == MYXML",
    true,
    (MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.children()[0].parent() == MYXML));

// The child is equal to child1 (but not the same object)
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.children()[0] == child1",
    true,
    (MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.children()[0] == child1));

// The child is a duplicate of child1
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.children()[0] === child1",
    true,
    (MYXML = new XML(xmlDoc), MYXML.insertChildBefore(null, child1), MYXML.children()[0] === child1));

var MYXML = new XML(xmlDoc);
MYXML.insertChildBefore(null, child1);

// !!@ this crashes Rhino's implementation
Assert.expectEq( "MYXML.insertChildBefore(child1, child2), MYXML.toString()",
    "<company><employee id=\"2\"><name>Sue</name></employee><employee id=\"1\"><name>John</name></employee></company>",
    (MYXML.insertChildBefore(child1, child2), MYXML.toString()));


var MYXML = new XML(xmlDoc);
MYXML.insertChildBefore(null, child1);

Assert.expectEq( "MYXML.insertChildBefore(MYXML.children()[0], child2), MYXML.toString()",
    "<company><employee id=\"2\"><name>Sue</name></employee><employee id=\"1\"><name>John</name></employee></company>",
    (MYXML.insertChildBefore(MYXML.children()[0], child2), MYXML.toString()));

MYXML = new XML(xmlDoc);
MYXML.insertChildBefore(null, child2);
MYXML.insertChildBefore(MYXML.children()[0], child1);

// !!@ this crashes Rhino's implementation
Assert.expectEq( "MYXML.insertChildBefore(child2, child3), MYXML.toString()",
    "<company><employee id=\"1\"><name>John</name></employee><employee id=\"3\"><name>Bob</name></employee><employee id=\"2\"><name>Sue</name></employee></company>",
    (MYXML.insertChildBefore(child2, child3), MYXML.toString()));

MYXML = new XML(xmlDoc);
MYXML.insertChildBefore(null, child2);
MYXML.insertChildBefore(MYXML.children()[0], child1);

Assert.expectEq( "MYXML.insertChildBefore(MYXML.children()[1], child3), MYXML.toString()",
    "<company><employee id=\"1\"><name>John</name></employee><employee id=\"3\"><name>Bob</name></employee><employee id=\"2\"><name>Sue</name></employee></company>",
    (MYXML.insertChildBefore(MYXML.children()[1], child3), MYXML.toString()));
    
MYXML = new XML(xmlDoc);

Assert.expectEq("MYXML.insertChildBefore(null, XMLList), MYXML.toString()",
             new XML(wholeString).toString(),
             (MYXML.insertChildBefore(null, allChildren), MYXML.toString()));
             
MYXML = new XML(xmlDoc);
MYXML.insertChildBefore(null, child3);

Assert.expectEq("MYXML.insertChildBefore(child3, XMLList), MYXML.toString()",
             new XML(wholeString).toString(),
             (MYXML.insertChildBefore(MYXML.children()[0], twoChildren), MYXML.toString()));
             
MYXML = new XML(xmlDoc);
MYXML.insertChildBefore(null, child1);

Assert.expectEq("MYXML.insertChildBefore(child1, \"string\"), MYXML.toString()",
         new XML("<company>string<employee id='1'><name>John</name></employee></company>").toString(),
         (MYXML.insertChildBefore(MYXML.children()[0], "string"), MYXML.toString()));
             
var a = <a><b><c/></b></a>;

try {
    a.b.c.insertChildBefore (null, a);
    result = a;
} catch (e1) {
    result = Utils.typeError(e1.toString());
}
Assert.expectEq("a = <a><b><c/></b></a>, a.b.c.insertChildBefore(null,a)", "TypeError: Error #1118", result);




END();
