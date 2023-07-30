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

START("11.2.1 - Property Accessors");


function convertToString(o:Object){
  return o.toString();
}

order = 
<order id="123456" timestamp="Mon Mar 10 2003 16:03:25 GMT-0800 (PST)">
    <customer>
        <firstname>John</firstname>
        <lastname>Doe</lastname>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
</order>;    

correct = 
<customer>
    <firstname>John</firstname>
    <lastname>Doe</lastname>
</customer>;

TEST(1, correct, order.customer);
TEST_XML(2, 123456, order.@id);

correct = 
<item>
    <description>Big Screen Television</description>
    <price>1299.99</price>
    <quantity>1</quantity>
</item>

TEST(3, correct, order.children()[1]);

correct = 
<customer>
    <firstname>John</firstname>
    <lastname>Doe</lastname>
</customer> +
<item>
    <description>Big Screen Television</description>
    <price>1299.99</price>
    <quantity>1</quantity>
</item>;


TEST(4, correct, order.*); 

correct = new XMLList();
correct += new XML("123456");
correct += new XML("Mon Mar 10 2003 16:03:25 GMT-0800 (PST)");
TEST(5, correct, order.@*);

order = <order>
        <customer>
            <firstname>John</firstname>
            <lastname>Doe</lastname>
        </customer>
        <item id="3456">
            <description>Big Screen Television</description>
            <price>1299.99</price>
            <quantity>1</quantity>
        </item>
        <item id="56789">
            <description>DVD Player</description>
            <price>399.99</price>
            <quantity>1</quantity>
        </item>
        </order>;

correct =
<description>Big Screen Television</description> +
<description>DVD Player</description>;

TEST(6, correct, order.item.description);

correct = new XMLList();
correct += new XML("3456");
correct += new XML("56789");
TEST(7, correct, order.item.@id);

correct =
<item id="56789">
    <description>DVD Player</description>
    <price>399.99</price>
    <quantity>1</quantity>
</item>

TEST(8, correct, order.item[1]);

correct = 
<description>Big Screen Television</description> +
<price>1299.99</price> +
<quantity>1</quantity> +
<description>DVD Player</description> +
<price>399.99</price> +
<quantity>1</quantity>;

TEST(9, correct, order.item.*);

correct=
<price>1299.99</price>;

TEST(10, correct, order.item.*[1]);

// get the first (and only) order [treating single element as a list]
order = <order>
        <customer>
            <firstname>John</firstname>
            <lastname>Doe</lastname>
        </customer>
        <item id="3456">
            <description>Big Screen Television</description>
            <price>1299.99</price>
            <quantity>1</quantity>
        </item>
        <item id="56789">
            <description>DVD Player</description>
            <price>399.99</price>
            <quantity>1</quantity>
        </item>
        </order>;


TEST(11, order, order[0]);

// Any other index should return undefined
TEST(12, undefined, order[1]);

var xml = "<order id = \"123456\"><c><f>John</f><l>Doe</l></c><i><desc>B</desc><p>1299.99</p><q>1</q></i><i><desc>A</desc><p>12.99</p><q>1</q></i></order>";

// XML object test cases

Assert.expectEq("XML.c.f:", "John", (o = new XML(xml), o.c.f.toString()));

Assert.expectEq("XML.c['f']:", "John", (o = new XML(xml), o.c['f'].toString()));

Assert.expectEq("XML.c.f[0]:", "John", (o = new XML(xml), o.c.f[0].toString()));

Assert.expectEq("XML.i[1].p:", "12.99", (o = new XML(xml), o.i[1].p.toString()));

Assert.expectEq("XML.i[1]['p]'", "12.99", (o = new XML(xml), o.i[1]['p'].toString()));


// High ASCII test
var xmlHighASCII = "<f><fname>Sören Lehmenkühler</fname></f>";

Assert.expectEq("High ASCII node value:", "Sören Lehmenkühler", (o = new XML(xmlHighASCII), o.fname.toString()));


// XMLList object test cases

Assert.expectEq("XMLList.c.f:", "John", (ol = new XMLList(xml), ol.c.f.toString()));

Assert.expectEq("XMLList.c[\"f\"]:", "John", (ol = new XMLList(xml), ol.c["f"].toString()));

Assert.expectEq("XMLList.c.f[0]:", "John", (o = new XMLList(xml), o.c.f[0].toString()));

Assert.expectEq("XMLList.c.f[0] = \"Peter\":", "Peter", (o = new XMLList(xml), o.c.f[0] = "Peter", o.c.f[0].toString()));

Assert.expectEq("XMLList.i[1].p:", "12.99", (ol = new XMLList(xml), ol.i[1].p.toString()));

Assert.expectEq("XMLList.i[1][\"p\"]:", "12.99", (ol = new XMLList(xml), ol.i[1]["p"].toString()));

Assert.expectEq("XMLList[1] = <a>b</a>", "b", (ol = new XMLList(), ol[1] = <a>b</a>, ol.toString())); 

Assert.expectEq("XMLList[1] = <a>b</a>; XMLList[0] = <c>d</c>", "d", (ol = new XMLList(), ol[1] = <a>b</a>, ol[0] = <c>d</c>, ol.toString())); 

Assert.expectEq("XMLList[0] = <a>b</a>; XMLList[1] = <c>d</c>", convertToString(new XMLList("<a>b</a><c>d</c>")), (ol = new XMLList(), ol[0] = <a>b</a>, ol[1] = <c>d</c>, ol).toString()); 


var x1 = new XML("<x><fname>a</fname><fname>b</fname><fname>c</fname></x>");
var y1 = x1.fname;

Assert.expectEq("x1.f == x1.f[0] + x1.f[1] + x1.f[2]", convertToString(x1.fname[0] + x1.fname[1] + x1.fname[2]),
x1.fname.toString());

// comparing XML and XMLList equivalents

Assert.expectEq("XML[0].fname[1] == XMLList[1]:", true, (y1 = x1.fname, (x1[0].fname[1] == y1[1])));

Assert.expectEq("XML[0].fname[0] == XMLList.fname[0]:", true, (y1 = new XMLList(x1), (x1[0].fname[0] == y1.fname[0])));

 
var hyphenatedXML = new XML("<a><b-c a='1'>blue</b-c><b-c a='2'>orange</b-c><b-c a='3'>yellow</b-c></a>");

Assert.expectEq("hyphenatedXML.[\"b-c\"]:", "orange", (hyphenatedXML["b-c"][1].toString()));

Assert.expectEq("hyphenatedXML.[\"b-c\"][1] = \"new color\":", "pink", (hyphenatedXML["b-c"][1] = "pink", hyphenatedXML["b-c"][1].toString()));

xL = <x a="aatr" b="batr">y</x>;

Assert.expectEq("x['*']", "y", xL['*'].toString());
Assert.expectEq("x['@*']", "aatrbatr", xL['@*'].toString());
Assert.expectEq("x['@a']", "aatr", xL['@a'].toString());

xL = <x xmlns:ns1="foo" xmlns:ns2="bar" ns1:prop='prop1' ns2:prop='prop2' prop='prop3'>some text</x>; 

Assert.expectEq("x1.@prop", "prop3", xL.@prop.toString());

function setNS1() {
	use namespace foo;
	Assert.expectEq("use namespace foo; x1.@prop", "prop1prop3", xL.@prop.toString());
}

function setNS2() {
	namespace foo2 = "bar"; 
	use namespace foo2;
	Assert.expectEq("use namespace foo2; x1.@prop", "prop2prop3", xL.@prop.toString());
}

function setNS3() {
	use namespace foo;
	namespace foo2 = "bar"; 
	use namespace foo2;
	Assert.expectEq("use namespace foo2; x1.@prop", "prop1prop2prop3", xL.@prop.toString());
}

namespace foo = "foo"; 

setNS1();

setNS2();

setNS3();

END();
