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

START("13.5.4.4 - XMLList child()");

//TEST(1, true, XMLList.prototype.hasOwnProperty("child"));

// Test with XMLList of size 0
x1 = new XMLList()
TEST(2, "xml", typeof(x1.child("bravo")));
TEST_XML(3, "", x1.child("bravo"));

// Test with XMLList of size 1
x1 += <alpha>one<bravo>two</bravo></alpha>;
TEST(4, "xml", typeof(x1.child("bravo")));
TEST_XML(5, "<bravo>two</bravo>", x1.child("bravo"));

x1 += <charlie><bravo>three</bravo></charlie>;
TEST(6, "xml", typeof(x1.child("bravo")));

correct = <><bravo>two</bravo><bravo>three</bravo></>;
TEST(7, correct, x1.child("bravo"));

// Test no match, null and undefined
TEST(8, "xml", typeof(x1.child("foobar")));
TEST_XML(9, "", x1.child("foobar"));

try {
  x1.child(null);
  SHOULD_THROW(10);
} catch (ex) {
  TEST(10, "TypeError", ex.name);
}

// Test numeric inputs
x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>two</charlie>
</alpha>;

TEST(12, <bravo>one</bravo>, x1.child(0));
TEST(13, <charlie>two</charlie>, x1.child(1));

var xmlDoc = "<MLB><Team>Giants</Team><City>San Francisco</City></MLB><MLB2><Team>Padres</Team><City>San Diego</City></MLB2>";

// propertyName as a string
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child('Team')", "<Team>Giants</Team>" + NL() + "<Team>Padres</Team>",
             (MYXML = new XMLList(xmlDoc), MYXML.child('Team').toString() ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child('Team') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.child('Team') instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child('Team') instanceof XML", false,
             (MYXML = new XMLList(xmlDoc), MYXML.child('Team') instanceof XML ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child('Team').length()", 2,
             (MYXML = new XMLList(xmlDoc), MYXML.child('Team').length()));
Assert.expectEq( "MYXML = new XMLList(null), MYXML.child('Team')", "",
             (MYXML = new XMLList(null), MYXML.child('Team').toString() ));
Assert.expectEq( "MYXML = new XMLList(undefined), MYXML.child('Team')", "",
             (MYXML = new XMLList(undefined), MYXML.child('Team').toString() ));
Assert.expectEq( "MYXML = new XMLList(), MYXML.child('Team')", "",
             (MYXML = new XMLList(), MYXML.child('Team').toString() ));

// propertyName as a numeric index
// !!@ doesn't work in Rhino. Should this return the 1st child (from 0th)
// of the MLB node which should be "San Francisco"
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child(1) instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.child(1) instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child(1)", "<City>San Francisco</City>" + NL() + "<City>San Diego</City>",
             (MYXML = new XMLList(xmlDoc), MYXML.child(1).toString() ));
Assert.expectEq( "MYXML = new XMLList(null), MYXML.child(1)", "",
             (MYXML = new XMLList(null), MYXML.child(1).toString() ));
Assert.expectEq( "MYXML = new XMLList(undefined), MYXML.child(1)", "",
             (MYXML = new XMLList(undefined), MYXML.child(1).toString() ));
Assert.expectEq( "MYXML = new XMLList(), MYXML.child(1)", "",
             (MYXML = new XMLList(), MYXML.child(1).toString() ));

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child(0) instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.child(0) instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child(0)", "<Team>Giants</Team>" + NL() + "<Team>Padres</Team>",
             (MYXML = new XMLList(xmlDoc), MYXML.child(0).toString() ));
Assert.expectEq( "MYXML = new XMLList(null), MYXML.child(0)", "",
             (MYXML = new XMLList(null), MYXML.child(0).toString() ));
Assert.expectEq( "MYXML = new XMLList(undefined), MYXML.child(0)", "",
             (MYXML = new XMLList(undefined), MYXML.child(0).toString() ));
Assert.expectEq( "MYXML = new XMLList(), MYXML.child(0)", "",
             (MYXML = new XMLList(), MYXML.child(0).toString() ));

// propertyName is invalid

// invalid propertyName
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child('DoesNotExist') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.child('DoesNotExist') instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child('DoesNotExist')", "",
             (MYXML = new XMLList(xmlDoc), MYXML.child('DoesNotExist').toString() ));
Assert.expectEq( "MYXML = new XMLList(null), MYXML.child('DoesNotExist')", "",
             (MYXML = new XMLList(null), MYXML.child('DoesNotExist').toString() ));
Assert.expectEq( "MYXML = new XMLList(undefined), MYXML.child('DoesNotExist')", "",
             (MYXML = new XMLList(undefined), MYXML.child('DoesNotExist').toString() ));
Assert.expectEq( "MYXML = new XMLList(), MYXML.child('DoesNotExist')", "",
             (MYXML = new XMLList(), MYXML.child('DoesNotExist').toString() ));

// invalid index
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child(8) instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.child(8) instanceof XMLList ));
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.child(8)", "",
             (MYXML = new XMLList(xmlDoc), MYXML.child(8).toString() ));
Assert.expectEq( "MYXML = new XMLList(null), MYXML.child(8)", "",
             (MYXML = new XMLList(null), MYXML.child(8).toString() ));
Assert.expectEq( "MYXML = new XMLList(undefined), MYXML.child(8)", "",
             (MYXML = new XMLList(undefined), MYXML.child(8).toString() ));
Assert.expectEq( "MYXML = new XMLList(), MYXML.child(8)", "",
             (MYXML = new XMLList(), MYXML.child(8).toString() ));

END();
