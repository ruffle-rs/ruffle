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
 

START("11.3.1 - Delete Operator");

order =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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

// Delete the customer address
correct =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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

delete order.customer.address;
TEST_XML(1, "", order.customer.address);
TEST(2, correct, order);

order =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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

// delete the custmomer ID
correct =
<order id="123456">
    <customer>
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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

delete order.customer.@id;
TEST_XML(3, "", order.customer.@id);
TEST(4, correct, order);

order =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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

// delete the first item price
correct =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <quantity>1</quantity>
    </item>
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

delete order.item.price[0];
TEST_XML(5, "", order.item[0].price);
TEST(6, <price>1299.99</price>, order.item.price[0]);
TEST(7, order, correct);

order =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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

// delete all the items
correct =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
</order>;

delete order.item;
TEST_XML(8, "", order.item);
TEST(9, correct, order);

order =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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


// delete all description tags with descendant operator
// is not supposed to do anything, see bug 149397
correct =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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
   

delete order..description;
TEST(10, correct, order);

order =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <customer id="456">
            <firstname>Mary</firstname>
            <lastname>Jones</lastname>
            <address>456 Foobar Ave.</address>
            <city>Bel Air</city>
            <state>CA</state>
    </customer>
</order>;

try {
    delete order.customer.(firstname == "John");
    result = order;
} catch (e1) {
    result = Utils.typeError(e1.toString());
}

Assert.expectEq("Delete an XMLList", "TypeError: Error #1119", result);

order =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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

// delete all id attributes
correct =
<order id="123456">
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
    <item>
        <description>DVD Player</description>
        <price>399.99</price>
        <quantity>1</quantity>
    </item>
</order>;
   

delete order.item.@id;
TEST(11, correct, order);

order =
<order id="123456">
  <blah>
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
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
  </blah>
</order>;

// delete all id attributes, using descendant operator
correct =
<order id="123456">
  <blah>
    <customer id="123">
        <firstname>John</firstname>
        <lastname>Doe</lastname>
        <address>123 Foobar Ave.</address>
        <city>Bellevue</city>
        <state>WA</state>
    </customer>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
    <item>
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <quantity>1</quantity>
    </item>
    <item>
        <description>DVD Player</description>
        <price>399.99</price>
        <quantity>1</quantity>
    </item>
  </blah>
</order>;
   

delete order..item.@id;
TEST(12, correct, order);

//default xml namespace = "http://someuri";
x1 = <x/>;
x1.a.b = "foo";
delete x1.a.b;
TEST_XML(10, "", x1.a.b);

var ns = new Namespace("");
x1.a.b = <b xmlns="">foo</b>;
TEST(11, "foo", x1.a.ns::b.toString());

delete x1.a.b;
TEST(12, "", x1.a.ns::b.toString());

delete x1.a.ns::b;
TEST_XML(13, "", x1.a.ns::b);

var y1;
x1 = new XML("<a><b><c>C</c><d>D</d></b></a>");
y1 = new XML("<a><b><c>C</c></b></a>");
 
Assert.expectEq("delete XML:", true, (delete x1.b.d, (x1 == y1)));


 
x1 = new XMLList("<d><a>A</a><b>B</b><c>C</c></d>");
y1 = new XMLList("<d><a>A</a><c>C</c></d>");

Assert.expectEq("delete XMLList:", true, (delete x1.b, (x1 == y1)));

END();
