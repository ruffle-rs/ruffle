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

START("13.5.1 - XMLList Constructor as Function");

x1 = XMLList();

TEST(1, "xml", typeof(x1));
TEST(2, true, x1 instanceof XMLList);

// Make sure it's not copied if it's an XMLList
x1 = new XMLList("<alpha>one</alpha><bravo>two</bravo>");


y1 = XMLList(x1);
TEST(3, x1 === y1, true);

x1 += <charlie>three</charlie>;
TEST(4, x1 === y1, false);

// Load from one XML type
x1 = XMLList(<alpha>one</alpha>);
TEST_XML(5, "<alpha>one</alpha>", x1);

// Load from Anonymous
x1 = XMLList(<><alpha>one</alpha><bravo>two</bravo></>);
correct = new XMLList();
correct += <alpha>one</alpha>;
correct += <bravo>two</bravo>;
TEST_XML(6, correct.toString(), x1);

// Load from Anonymous as string
x1 = XMLList(<><alpha>one</alpha><bravo>two</bravo></>);
correct = new XMLList();
correct += <alpha>one</alpha>;
correct += <bravo>two</bravo>;
TEST_XML(7, correct.toString(), x1);

// Load from single textnode
x1 = XMLList("foobar");
TEST_XML(8, "foobar", x1);

x1 = XMLList(7);
TEST_XML(9, "7", x1);

// Undefined and null should behave like ""
x1 = XMLList(null);
TEST_XML(10, "", x1);

x1 = XMLList(undefined);
TEST_XML(11, "", x1);

XML.prettyPrinting = false;

var thisXML = "<TEAM>Giants</TEAM><CITY>San Francisco</CITY>";

// value is null
Assert.expectEq( "XMLList(null).toString()", "", XMLList(null).toString() );
Assert.expectEq( "typeof XMLList(null)", "xml", typeof XMLList(null) );
Assert.expectEq( "XMLList(null) instanceof XMLList", true, XMLList(null) instanceof XMLList);

// value is undefined
Assert.expectEq( "XMLList(undefined).toString()", "", XMLList(undefined).toString() );
Assert.expectEq( "typeof XMLList(undefined)", "xml", typeof XMLList(undefined) );
Assert.expectEq( "XMLList(undefined) instanceof XMLList", true, XMLList(undefined) instanceof XMLList);

// value is not supplied
Assert.expectEq( "XMLList().toString()", "", XMLList().toString() );
Assert.expectEq( "typeof XMLList()", "xml", typeof XMLList() );
Assert.expectEq( "XMLList() instanceof XMLList", true, XMLList() instanceof XMLList);

// value is supplied (string)
Assert.expectEq( "XMLList(thisXML).toString()",
    "<TEAM>Giants</TEAM>" + NL() + "<CITY>San Francisco</CITY>",
    XMLList(thisXML).toString() );
Assert.expectEq( "typeof XMLList(thisXML)", "xml", typeof XMLList(thisXML) );

// value is supplied (xmlList)
var xl = new XMLList ("<foo>bar></foo><foo2>bar></foo2>");
Assert.expectEq( "XMLList(xl) === xl", true, XMLList(xl) === xl);

// value is supplied (xml)
var x1 = new XML ("<foo>bar></foo>");
Assert.expectEq( "XMLList(x1).length()", 1, XMLList(x1).length());
Assert.expectEq( "XMLList(x1).contains(x1)", true, XMLList(x1)[0].contains(x1));

END();
