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

START("13.4.4.27 - XML parent()");

//TEST(1, true, XML.prototype.hasOwnProperty("parent"));
    
var x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>
        <bravo>two</bravo>
    </charlie>
</alpha>;

var y1 = x1.bravo;

TEST(2, x1, y1.parent());

TEST(3, undefined, x1.parent());

x1 = new XML("<XML></XML>");
var a = new XML("text node");
var b = new XML("<foo>bar</foo>");

Assert.expectEq ("MYXML = new('<XML></XML>'), MYXML.parent()", undefined, x1.parent());
Assert.expectEq ("MYXML = new('text node'), MYXML.parent()", undefined, a.parent());
Assert.expectEq ("MYXML = new('<foo>bar</foo>'), MYXML.parent()", undefined, b.parent());

x1.appendChild (a);
x1.appendChild (b);


// Text node is a special case
Assert.expectEq ("a - orphan node after x.appendChild, a.parent()", undefined, a.parent());
Assert.expectEq ("b - orphan node after x.appendChild, b.parent()", x1, b.parent());

Assert.expectEq ("x1.children()[0].parent()", x1, x1.children()[0].parent());
Assert.expectEq ("x1.children()[1].parent()", x1, x1.children()[0].parent());
Assert.expectEq ("x1.children()[0].parent() === x1", true, (x1.children()[0].parent() === x1));
Assert.expectEq ("x1.children()[1].parent() === x1", true, (x1.children()[0].parent() === x1));

var xDoc = "<company><employee id='1'><name1>John</name1> <city>California</city> </employee><employee id='2'><name1>Mary</name1> <city>Texas</city> </employee></company>";

Assert.expectEq( "x1 = new XML(xDoc), x1.employee[0].parent() == x1", true, (x1 = new XML(xDoc), x1.employee[0].parent() == x1));
Assert.expectEq( "x1 = new XML(xDoc), x1.employee.name1[0].parent() == x1.employee[0]", true, (x1 = new XML(xDoc), x1.employee.name1[0].parent() == x1.employee[0]));
Assert.expectEq( "x1 = new XML(xDoc), x1.employee[0].attribute('id').parent() === x1.employee[0]", true, (x1 = new XML(xDoc), x1.employee[0].attribute('id').parent() === x1.employee[0]));

Assert.expectEq( "x1 = new XML(xDoc), x1.employee[1].parent() == x1", true, (x1 = new XML(xDoc), x1.employee[1].parent() == x1));
Assert.expectEq( "x1 = new XML(xDoc), x1.employee.name1[1].parent() == x1.employee[0]", true, (x1 = new XML(xDoc), x1.employee.name1[1].parent() == x1.employee[1]));
Assert.expectEq( "x1 = new XML(xDoc), x1.employee[1].attribute('id').parent() === x1.employee[1]", true, (x1 = new XML(xDoc), x1.employee[1].attribute('id').parent() === x1.employee[1]));

XML.ignoreComments = false;
XML.ignoreProcessingInstructions = false;
xDoc = "<simple><!-- comment --><?x-stylesheet href=\"classic.xsl\" type=\"text/x\"?></simple>";

// Tests comments and PI nodes
Assert.expectEq( "x1 = new XML(xDoc), x1.children()[0].parent() == x1", true, (x1 = new XML(xDoc), x1.children()[0].parent() == x1));
Assert.expectEq( "x1 = new XML(xDoc), x1.children()[1].parent() == x1", true, (x1 = new XML(xDoc), x1.children()[1].parent() == x1));
Assert.expectEq( "x1 = new XML(xDoc), x1.children().parent() == x1", true, (x1 = new XML(xDoc), x1.children().parent() == x1));


END();
