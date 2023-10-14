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

START("11.2.2 - Function Calls");


function convertToString(o:Object){
  return o.toString();
}

rectangle = <rectangle>
            <x>50</x>
            <y>75</y>
            <length>20</length>
            <width>30</width>
            </rectangle>;


TEST(1, 1, rectangle.length());

TEST(2, <length>20</length>, rectangle.length);

shipto = <shipto>
         <name>Fred Jones</name>
         <street>123 Foobar Ave.</street>
         <citystatezip>Redmond, WA, 98008</citystatezip>
         </shipto>;


upperName = shipto.name.toUpperCase();
TEST(3, "FRED JONES", upperName);

upperName = shipto.name.toString().toUpperCase();
TEST(4, "FRED JONES", upperName);
upperName = shipto.name.toUpperCase();
TEST(5, "FRED JONES", upperName);

citystatezip = shipto.citystatezip.split(", ");
state = citystatezip[1];
TEST(6, "WA", state);
zip = citystatezip[2];
TEST(7, "98008", zip);


citystatezip = shipto.citystatezip.toString().split(", ");
state = citystatezip[1];
TEST(8, "WA", state);
zip = citystatezip[2];
TEST(9, "98008", zip);

foo = <top><apple>hello</apple></top>;
var1 = foo.apple;
foo.apple = "moi";
TEST(10, <apple>moi</apple>, var1);

// Test method name/element name conflicts

x1 =
<alpha>
    <name>Foo</name>
    <length>Bar</length>
</alpha>;

TEST(11, QName("alpha"), x1.name());
TEST(12, <length>Bar</length>, x1.length);
TEST(13, 1, x1.length());
TEST(14, x1, x1.(length == "Bar"));

x1.name = "foobar";

TEST(15, <name>foobar</name>, (x1.name));
TEST(16, QName("alpha"), (x1.name()));

var xml = "<person><name>Bubba</name></person>";

Assert.expectEq("person.name:", "Bubba", (x1 = new XML(xml), x1.name.toString()));

Assert.expectEq("person.name():", "person", (x1 = new XML(xml), x1.name().toString()));


xml = "<i><length>5</length><width>30</width></i>";

Assert.expectEq("i.length:", "5", (x1 = new XML(xml), x1.length.toString()));

Assert.expectEq("i.length():", 1, (x1 = new XML(xml), x1.length()));


xml = "<xml><parent><i>a</i><i>b</i><i>c</i></parent></xml>";
var p = new XMLList(xml).parent;

Assert.expectEq("x.parent:", p.toString(), (x1 = new XML(xml), x1.parent).toString());

Assert.expectEq("x.parent():", undefined, (x1 = new XML(xml), x1.parent()));

Assert.expectEq("x.parent.parent():", x1.toString(), (x1 = new XML(xml), x1.parent.parent()).toString());



END();
