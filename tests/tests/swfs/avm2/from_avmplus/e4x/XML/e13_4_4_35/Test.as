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
 
function typeError(){
  // nothing here
}

START("13.4.4.35 - setName");

//TEST(1, true, XML.prototype.hasOwnProperty("setName"));

x1 =
<alpha>
    <bravo>one</bravo>
</alpha>;

correct =
<charlie>
    <bravo>one</bravo>
</charlie>;

x1.setName("charlie");

TEST(2, correct, x1);

x1 =
<ns:alpha xmlns:ns="http://foobar/">
    <ns:bravo>one</ns:bravo>
</ns:alpha>;

correct =
<charlie xmlns:ns="http://foobar/">
    <ns:bravo>one</ns:bravo>
</charlie>;

x1.setName("charlie");

TEST(3, correct, x1);

x1 =
<ns:alpha xmlns:ns="http://foobar/">
    <ns:bravo>one</ns:bravo>
</ns:alpha>;

correct =
<ns:charlie xmlns:ns="http://foobar/">
    <ns:bravo>one</ns:bravo>
</ns:charlie>;

x1.setName(new QName("http://foobar/", "charlie"));

TEST(4, correct, x1);

XML.prettyPrinting = false;
var xmlDoc = "<company><employee id='1'><name>John</name> <city>California</city> </employee></company>";

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setName('employees'),MYXML.name().toString()",
    "employees",
    (MYXML = new XML(xmlDoc),MYXML.setName('employees'), MYXML.name().toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setName(new QName('employees')),MYXML.name().toString()",
    "employees",
    (MYXML = new XML(xmlDoc),MYXML.setName(new QName('employees')), MYXML.name().toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setName(new QName('ns', 'employees')),MYXML.name().toString()",
    "ns::employees",
    (MYXML = new XML(xmlDoc),MYXML.setName(new QName('ns', 'employees')), MYXML.name().toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.setName('employees'),MYXML.toString()",
    "<employees><employee id=\"1\"><name>John</name><city>California</city></employee></employees>",
    (MYXML = new XML(xmlDoc),MYXML.setName('employees'), MYXML.toString()));

// Calling setName() on an attribute
Assert.expectEq("MYXML = new XML(xmlDoc), MYXML.employee.@id.setName('num')", "num", (MYXML = new XML(xmlDoc), MYXML.employee.@id.setName("num"), MYXML.employee.@num.name().toString()));

var TYPEERROR = "TypeError: Error #";
function typeError( str ){
    return str.slice(0,TYPEERROR.length+4);
}
MYXML = new XML(xmlDoc);
MYXML.employee.@id.setName("num");

try {
    MYXML.employee.@id.name();
    result = "no error";
} catch (e1) {
    result = typeError(e1.toString());
}

Assert.expectEq("MYXML = new XML(xmlDoc), MYXML.employee.@id.setName(\"num\"),MYXML.employee.@id.name())", "TypeError: Error #1086", result);
x1 =
<foo:alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
    <foo:bravo attr="1">one</foo:bravo>
</foo:alpha>;

ns = new Namespace("foo", "http://foo/");
correct = <foo:alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
    <foo:bravo foo="1">one</foo:bravo>
</foo:alpha>;

Assert.expectEq("Calling setName() on an attribute with same name as namespace", "|"+correct+"|", "|"+(x1.ns::bravo.@attr.setName("foo"), x1)+"|");

// throws Rhino exception - bad name
MYXML = new XML(xmlDoc);
try {
    MYXML.setName('@employees');
    result = " no error";
} catch (e2) {
    result = typeError(e2.toString());
}
Assert.expectEq("MYXML.setName('@employees')", "TypeError: Error #1117", result);

try {
    MYXML.setName('!hi');
    result = " no error";
} catch (e3) {
    result = typeError(e3.toString());
}
Assert.expectEq("MYXML.setName('!hi')", "TypeError: Error #1117", result);

try {
    MYXML.setName('1+1=5');
    result = " no error";
} catch (e4) {
    result = typeError(e4.toString());
}
Assert.expectEq("MYXML.setName('1+1=5')", "TypeError: Error #1117", result);

try {
    MYXML.setName('555');
    result = " no error";
} catch (e5) {
    result = typeError(e5.toString());
}
Assert.expectEq("MYXML.setName('555')", "TypeError: Error #1117", result);


try {
    MYXML.setName('1love');
    result = " no error";
} catch (e6) {
    result = typeError(e6.toString());
}
Assert.expectEq("MYXML.setName('1love')", "TypeError: Error #1117", result);

try {
    MYXML.setName('*');
    result = " no error";
} catch (e7) {
    result = typeError(e7.toString());
}
Assert.expectEq("MYXML.setName('*')", "TypeError: Error #1117", result);



END();
