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
 
START("11.1.1 - Attribute Identifiers");
x1 =
<alpha>
    <bravo attr1="value1" ns:attr1="value3" xmlns:ns="http://someuri">
        <charlie attr1="value2" ns:attr1="value4"/>
    </bravo>
</alpha>
    
TEST_XML(1, "value1", x1.bravo.@attr1);
TEST_XML(2, "value2", x1.bravo.charlie.@attr1);

correct = new XMLList();
correct += new XML("value1");
correct += new XML("value2");
TEST(3, correct, x1..@attr1);

n = new Namespace("http://someuri");
TEST_XML(4, "value3", x1.bravo.@n::attr1);
TEST_XML(5, "value4", x1.bravo.charlie.@n::attr1);

correct = new XMLList();
correct += new XML("value3");
correct += new XML("value4");
TEST(6, correct, x1..@n::attr1);

q = new QName("attr1");
q2 = new QName(q, "attr1");

TEST(7.3, "attr1", q.toString());
TEST(7.4, "attr1", q2.toString());

q = new QName(n, "attr1");
q2 = new QName(q, "attr1");
TEST(7, correct, x1..@[q]);
TEST(7.1, "http://someuri::attr1", q.toString());
TEST(7.2, "http://someuri::attr1", q2.toString());

correct = new XMLList();
correct += new XML("value1");
correct += new XML("value3");
correct += new XML("value2");
correct += new XML("value4");
TEST(8, correct, x1..@*::attr1);

TEST_XML(9, "value1", x1.bravo.@["attr1"]);
TEST_XML(10, "value3", x1.bravo.@n::["attr1"]);
TEST_XML(11, "value3", x1.bravo.@[q]);
TEST_XML(12, "value3", x1.bravo.@[q2]);


y = <ns:attr1 xmlns:ns="http://someuri"/>
q3 = y.name();

Assert.expectEq("q3 = y.name()", "http://someuri::attr1", q3.toString());
Assert.expectEq("x1.bravo.@[q3]", "|"+new XML("value3")+"|", "|"+x1.bravo.@[q3]+"|");


var xml1 = "<c><color c='1'>pink</color><color c='2'>purple</color><color c='3'>orange</color></c>";
var xml2 = "<animals><a a='dog'>Spot</a><a a='fish'>Marcel</a><a a='giraffe'>Virginia</a></animals>";
var xml3 = "<flowers><flower type='tulip' attr-with-hyphen='got it'><color>yellow</color></flower></flowers>";
var xml4 = "<ns1:a xmlns:ns1=\"http://yo-raps.tv\"><ns1:b attr=\"something\">rainbow</ns1:b></ns1:a>";

try {
    var xml5 = <x><myTag myAttrib="has an apostrophe' in it"/></x>;
    var res = "no exception";
    Assert.expectEq("Attribute with apostrophe in it", "has an apostrophe' in it", xml5.myTag.@myAttrib.toString());
} catch (e1) {
    var res = "exception";
} finally {
    // Needs to be fixed when bug 133471 is fixed
    Assert.expectEq("Attribute with apostrophe in it", "no exception", res);
}
    

var placeHolder = "c";

var ns1 = new Namespace('yo', 'http://yo-raps.tv');
var ns2 = new Namespace('mo', 'http://maureen.name');

Assert.expectEq("x1.node1[i].@attr", "1",
           ( x1 = new XML(xml1), x1.color[0].@c.toString()));

Assert.expectEq("x1.node1[i].@attr = \"new value\"", "5",
           ( x1 = new XML(xml1), x1.color[0].@c = "5", x1.color[0].@c.toString()));

Assert.expectEq("x1.node1[i].@[placeHolder]", "1",
           ( x1 = new XML(xml1), x1.color[0].@[placeHolder].toString()));

Assert.expectEq("x1.node1[i].@[placeHolder] = \"new value\"", "5",
           ( x1 = new XML(xml1), x1.color[0].@[placeHolder] = "5", x1.color[0].@[placeHolder].toString()));

Assert.expectEq("x1.node1[i].@attr", "giraffe",
           ( x1 = new XML(xml2), x1.a[2].@a.toString()));

Assert.expectEq("x1.node1[i].@attr = \"new value\"", "hippopotamus",
           ( x1 = new XML(xml2), x1.a[2].@a = "hippopotamus", x1.a[2].@a.toString()));

Assert.expectEq("x1.node1.@[attr-with-hyphen]", "got it",
           ( x1 = new XML(xml3), x1.flower.@["attr-with-hyphen"].toString()));

Assert.expectEq("x1.node1.@[attr-with-hyphen] = \"new value\"", "still got it",
           ( x1 = new XML(xml3), x1.flower.@["attr-with-hyphen"] = "still got it", x1.flower.@["attr-with-hyphen"].toString()));
           
Assert.expectEq("x1.namespace1::node1.@attr", "something",
           ( x1 = new XML(xml4), x1.ns1::b.@attr.toString()));

Assert.expectEq("x1.namespace1::node1.@attr = \"new value\"", "something else",
           ( x1 = new XML(xml4), x1.ns1::b.@attr = "something else", x1.ns1::b.@attr.toString()));
           

var ns = new Namespace("foo");
var y1 = <y xmlns:ns="foo" a="10" b="20" ns:c="30" ns:d="40"/>;
var an = 'a';

Assert.expectEq("y1.@a", "10", y1.@a.toString());

Assert.expectEq("y1.@[an]", "10", y1.@[an].toString());

Assert.expectEq("y1.@*", "10203040", y1.@*.toString());  // Rhino bug: doesn't include qualified attributes

Assert.expectEq("y1.@ns::*", 2, y1.@ns::*.length());

var z = <y xmlns:ns="foo" a="10" b="20" ns:c="30" ns:d="40"/>;
Assert.expectEq("y1.@b", "200", (z.@b = 200, z.@b.toString()));

Assert.expectEq("y1.@*", "103040", (delete y1.@b, y1.@*.toString()));

// Adding for bug 117159
var element:XML = new XML(<element function="foo"/>);
Assert.expectEq("Reserved keyword used as attribute name", "foo", element.@["function"].toString());

var xmlObj = new XML ();
xmlObj = XML ('<elem attr1="firstAttribute"></elem>');

try {
    e = xmlObj.(@nonExistentAttribute == "nonExistent");
    result = e;
} catch (e2) {
    result = Utils.referenceError(e2.toString());
}
Assert.expectEq("Access non-existent attribute", "ReferenceError: Error #1065", result);

END();
