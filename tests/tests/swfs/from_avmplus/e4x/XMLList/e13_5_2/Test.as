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

START("13.5.2 - XMLList Constructor");

x1 = new XMLList();
TEST(1, "xml", typeof(x1));
TEST(2, true, x1 instanceof XMLList);

// Load from another XMLList
// Make sure it is copied if it's an XMLList
x1 =
<>
    <alpha>one</alpha>
    <bravo>two</bravo>
</>;

y1 = new XMLList(x1);

x1 += <charlie>three</charlie>;

TEST(3, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", y1.toString());
   
// Load from one XML type
x1 = new XMLList(<alpha>one</alpha>);
TEST_XML(4, "<alpha>one</alpha>", x1);

// Load from Anonymous
x1 = new XMLList(<><alpha>one</alpha><bravo>two</bravo></>);
TEST(5, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", x1.toString());

// Load from Anonymous as string
x1 = new XMLList(<><alpha>one</alpha><bravo>two</bravo></>);
TEST(6, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>", x1.toString());

// Load from single textnode
x1 = new XMLList("foobar");
TEST_XML(7, "foobar", x1);

x1 = XMLList(7);
TEST_XML(8, 7, x1);

// Undefined and null should behave like ""
x1 = new XMLList(null);
TEST_XML(9, "", x1);

x1 = new XMLList(undefined);
TEST_XML(10, "", x1);



function convertToString(o:Object){
  return o.toString();
}

var thisXML = "<TEAM>Giants</TEAM><CITY>San Francisco</CITY>";

// value is null
Assert.expectEq( "typeof new XMLList(null)", "xml", typeof new XMLList(null) );
Assert.expectEq( "new XMLList(null) instanceof XMLList", true, new XMLList(null) instanceof XMLList);
Assert.expectEq( "(new XMLList(null).length())", 0, (new XMLList(null)).length());
Assert.expectEq( "MYOB = new XMLList(null); MYOB.toString()", "",
             (MYOB = new XMLList(null), MYOB.toString(), MYOB.toString()) );

// value is undefined
Assert.expectEq( "typeof new XMLList(undefined)", "xml", typeof new XMLList(undefined) );
Assert.expectEq( "new XMLList(undefined) instanceof XMLList", true, new XMLList(undefined) instanceof XMLList);
Assert.expectEq( "(new XMLList(undefined).length())", 0, (new XMLList(undefined)).length());
Assert.expectEq( "MYOB = new XMLList(undefined); MYOB.toString()", "",
             (MYOB = new XMLList(undefined), MYOB.toString(), MYOB.toString()) );

// value is not supplied
Assert.expectEq( "typeof new XMLList()", "xml", typeof new XMLList() );
Assert.expectEq( "new XMLList() instanceof XMLList", true, new XMLList() instanceof XMLList);
Assert.expectEq( "(new XMLList().length())", 0, (new XMLList()).length());
Assert.expectEq( "MYOB = new XMLList(); MYOB.toString()", "",
             (MYOB = new XMLList(), MYOB.toString(), MYOB.toString()) );

//value is a number
Assert.expectEq( "typeof new XMLList(5)", "xml", typeof new XMLList(5) );
Assert.expectEq( "new XMLList(5) instanceof XMLList", true, new XMLList(5) instanceof XMLList);
Assert.expectEq( "(new XMLList(5).length())", 1, (new XMLList(5)).length());
Assert.expectEq( "MYOB = new XMLList(5); MYOB.toString()", "5",
             (MYOB = new XMLList(5), MYOB.toString(), MYOB.toString()) );
Assert.expectEq( "MYOB = new XMLList(5); MYOB.toXMLString()", "5",
             (MYOB = new XMLList(5), MYOB.toXMLString(), MYOB.toXMLString()) );

//value is

// value is supplied
XML.prettyPrinting = false;
Assert.expectEq( "typeof new XMLList(thisXML)", "xml", typeof new XMLList(thisXML) );
Assert.expectEq( "new XMLList(thisXML) instanceof XMLList", true, new XMLList(thisXML) instanceof XMLList);
Assert.expectEq( "(new XMLList(thisXML).length())", 2, (new XMLList(thisXML)).length());
Assert.expectEq( "MYOB = new XMLList(thisXML); MYOB.toString()",
            "<TEAM>Giants</TEAM>" + NL() + "<CITY>San Francisco</CITY>",
             (MYOB = new XMLList(thisXML), MYOB.toString(), MYOB.toString()) );

// Strongly typed XMLList objects
var MYXML1:XMLList = new XMLList(thisXML);
Assert.expectEq("var MYXML:XMLList = new XMLList(\"<a>b</a><c>d</c>\");", new XMLList(thisXML).toString(), MYXML1.toString());

var MYXML2:XMLList = new XMLList(<><TEAM>Giants</TEAM><CITY>San Francisco</CITY></>);
Assert.expectEq("var MYXML:XMLList = new XMLList(<a>b</a><c>d</c>);", new XMLList(thisXML).toString(), MYXML2.toString());

var MYXML3:XMLList = <><TEAM>Giants</TEAM><CITY>San Francisco</CITY></>;
Assert.expectEq("var MYXML:XMLList = <><a>b</a><c>d</c></>;", new XMLList(thisXML).toString(), MYXML3.toString());

var MYXML4:XMLList = new XMLList();
Assert.expectEq("var MYXML:XMLList = new XMLList()", convertToString(new XMLList()), MYXML4.toString());

var MYXML5:XMLList = new XMLList(5);
Assert.expectEq("var MYXML:XMLList = new XMLList(5)", "5", MYXML5.toString());



END();
