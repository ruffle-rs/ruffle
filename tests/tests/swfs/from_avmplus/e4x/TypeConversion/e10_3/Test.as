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

START("10.3 - toXML");

var x1;

// boolean
x1 = new Boolean(true);
TEST_XML(1, "true", new XML(x1));

// number
x1 = new Number(123);
TEST_XML(2, "123", new XML(x1));

// String
x1 = new String("<alpha><bravo>one</bravo></alpha>");
TEST(3, <alpha><bravo>one</bravo></alpha>, new XML(x1));

// XML
x1 = new XML("<alpha><bravo>one</bravo></alpha>");
TEST(4, <alpha><bravo>one</bravo></alpha>, new XML(x1));

// XMLList
x1 = new XMLList("<alpha><bravo>one</bravo></alpha>");
TEST(5, <alpha><bravo>one</bravo></alpha>, new XML(x1));

try {
    x1 = new XMLList(<alpha>one</alpha> + <bravo>two</bravo>);
    new XML(x1);
    SHOULD_THROW(6);
} catch (ex) {
    TEST(6, "TypeError", ex.name);
}
/*
// Undefined

try
{
    ToXML(undefined);
    Assert.expectEq( "ToXML(undefined) :", true, false );
}
catch (e)
{
    Assert.expectEq( "ToXML(undefined) :", true, true );
}


// Null

try
{
    ToXML(null);
    Assert.expectEq( "ToXML(null)      :", true, false );
}
catch (e)
{
    Assert.expectEq( "ToXML(null)      :", true, true );
}


// Boolean

var xt = "<parent xmlns=''>true</parent>";
var xf = "<parent xmlns=''>false</parent>";

Assert.expectEq( "ToXML(true)      :", true, (ToXML(true)==xt) );
Assert.expectEq( "ToXML(false)     :", true, (ToXML(false)==xf) );


// Number

var xn = "<parent xmlns=''>1234</parent>";

Assert.expectEq( "ToXML(1234)      :", true, (ToXML(1234)==xn) );


// XML

var x1 = new XML("<a><b>A</b></a>");

Assert.expectEq( "ToXML(XML)       :", true, (ToXML(x1)==x1) );


// XMLList

x1 = new XML("<a>A</a>");

Assert.expectEq( "ToXML(XMLList)   :", true, (ToXML(x1)=="A") );


// XMLList - XMLList contains more than one property

x1 = <a>A</a>
    <b>B</b>
    <c>C</c>;

try
{
    ToXML(x);
    Assert.expectEq( "ToXML(XMLList)   :", true, false );
}
catch (e)
{
    Assert.expectEq( "ToXML(XMLList)   :", true, true );
}


// Object

var a = new Array();

try
{
    ToXML(a);
    Assert.expectEq( "ToXML(Object)    :", true, false );
}
catch (e)
{
    Assert.expectEq( "ToXML(Object)    :", true, true );
}
*/
END();
