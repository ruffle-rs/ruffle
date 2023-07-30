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

START("11.6.2 - XMLList Assignment");

// Set the name of the only customer in the order to Fred Jones
order =
<order>
    <customer>
        <name>John Smith</name>
    </customer>
    <item id="1">
        <description>Big Screen Television</description>
        <price>1299.99</price>
    </item>
    <item id="2">
        <description>DVD Player</description>
        <price>399.99</price>
    </item>
</order>;
    
correct =
<order>
    <customer>
        <name>Fred Jones</name>
    </customer>
    <item id="1">
        <description>Big Screen Television</description>
        <price>1299.99</price>
    </item>
    <item id="2">
        <description>DVD Player</description>
        <price>399.99</price>
    </item>
</order>;

//order.customer.name = "Fred Jones";
//TEST(1, correct, order);

// Replace all the hobbies for the only customer in the order
order =
<order>
    <customer>
        <name>John Smith</name>
        <hobby>Biking</hobby>
    </customer>
    <item id="1">
        <description>Big Screen Television</description>
        <price>1299.99</price>
    </item>
    <item id="2">
        <description>DVD Player</description>
        <price>399.99</price>
    </item>
</order>;

correct =
<order>
    <customer>
        <name>John Smith</name>
        <hobby>shopping</hobby>
    </customer>
    <item id="1">
        <description>Big Screen Television</description>
        <price>1299.99</price>
    </item>
    <item id="2">
        <description>DVD Player</description>
        <price>399.99</price>
    </item>
</order>;

order.customer.hobby = "shopping"
TEST(2, correct, order);

// Attempt to set the sale date of the item.  Throw an exception if more than 1 item exists.
order =
<order>
    <customer>
        <name>John Smith</name>
    </customer>
    <item id="1">
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <saledate>01-05-2002</saledate>
    </item>
</order>;

correct =
<order>
    <customer>
        <name>John Smith</name>
    </customer>
    <item id="1">
        <description>Big Screen Television</description>
        <price>1299.99</price>
        <saledate>05-07-2002</saledate>
    </item>
</order>;

order.item.saledate = "05-07-2002"
TEST(3, correct, order);

order =
<order>
    <customer>
        <name>John Smith</name>
        <hobby>Biking</hobby>
    </customer>
    <item id="1">
        <description>Big Screen Television</description>
        <price>1299.99</price>
    </item>
    <item id="2">
        <description>DVD Player</description>
        <price>399.99</price>
    </item>
</order>;

try {
    order.item.saledate = "05-07-2002";
    SHOULD_THROW(4);
} catch (ex) {
    TEST(4, "TypeError", ex.name);
}

// Replace all the employee's hobbies with their new favorite pastime
emps =
<employees>
    <employee id = "1">
        <name>John</name>
        <age>20</age>
        <hobby>skiing</hobby>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
        <hobby>running</hobby>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
        <hobby>Biking</hobby>
    </employee>
</employees>;

correct =
<employees>
    <employee id = "1">
        <name>John</name>
        <age>20</age>
        <hobby>skiing</hobby>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
        <hobby>running</hobby>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
        <hobby>working</hobby>
    </employee>
</employees>;

//emps.employee.(@id == 3).hobby = "working";
//TEST(5, correct, emps);

// Replace the first employee with George
emps =
<employees>
    <employee id = "1">
        <name>John</name>
        <age>20</age>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
    </employee>
</employees>;

correct =
<employees>
    <employee id = "4">
        <name>George</name>
        <age>27</age>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
    </employee>
</employees>;

emps.employee[0] = <employee id="4"><name>George</name><age>27</age></employee>;
TEST(6, emps, correct);

// Add a new employee to the end of the employee list
emps =
<employees>
    <employee id = "1">
        <name>John</name>
        <age>20</age>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
    </employee>
</employees>;

correct =
<employees>
    <employee id = "1">
        <name>John</name>
        <age>20</age>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
    </employee>
    <employee id="4">
        <name>Frank</name>
        <age>39</age>
    </employee>
</employees>;

emps.employee += <employee id="4"><name>Frank</name><age>39</age></employee>;
TEST(7, correct, emps);

// Add a new employee to the end of the employee list
emps =
<employees>
    <employee id = "1">
        <name>John</name>
        <age>20</age>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
    </employee>
</employees>;

correct =
<employees>
    <employee id = "1">
        <name>John</name>
        <age>20</age>
    </employee>
    <employee id = "2">
        <name>Sue</name>
        <age>30</age>
    </employee>
    <employee id = "3">
        <name>Ted</name>
        <age>35</age>
    </employee>
    <employee id="4">
        <name>Frank</name>
        <age>39</age>
    </employee>
</employees>;

emps.employee[emps.employee.length()] = <employee id="4"><name>Frank</name><age>39</age></employee>;
TEST(7, correct, emps);


// property name is NOT array index

var x1 = new XML("<a><b><c>A</c><d>B</d></b></a>");

x1.b.d = "C";

var y1 = new XML("<a><b><c>A</c><d>C</d></b></a>");

Assert.expectEq( "property name is not array index :", true, (x1==y1) );


x1 = new XML("<a><b><c>A</c><d>B</d></b></a>");

x1.v.b.d = "C";

y1 = new XML("<a><b><c>A</c><d>B</d></b><v><b><d>C</d></b></v></a>");

Assert.expectEq("Adding new nested node at root: " , true, (x1 == y1));


// property name is array index

x1 = new XML("<a><b><c>A0</c><d>B0</d></b><b><c>A1</c><d>B1</d></b><b><c>A2</c><d>B2</d></b></a>");

x1.b[1] = new XML("<b><c>A3</c><d>B3</d></b>");

y1 = new XML("<a><b><c>A0</c><d>B0</d></b><b><c>A3</c><d>B3</d></b><b><c>A2</c><d>B2</d></b></a>");

Assert.expectEq( "property name exists in XMLList     :", true, (x1==y1) );


// property name does NOT exist in XMLList

x1.b[3] = new XML("<b><c>A4</c><d>B4</d></b>");

y1 = new XML("<a><b><c>A0</c><d>B0</d></b><b><c>A3</c><d>B3</d></b><b><c>A2</c><d>B2</d></b><b><c>A4</c><d>B4</d></b></a>");

Assert.expectEq( "property name does NOT exist in XMLList :", true, (x1==y1) );


// property is XML with non-null parent

x1 = new XML("<a><b><c>A0</c><d>B0</d></b><b><c>A1</c><d>B1</d></b><b><c>A2</c><d>B2</d></b></a>");

x1.b = new XMLList("<b>A</b><c>B</c>");

y1 = new XML("<a><b>A</b><c>B</c></a>");

Assert.expectEq( "property is XML with non-null parent    :", true, (x1==y1) );


// AssignmentExpression = XML value

x1 = new XML("<a><b><c>A0</c><d>B0</d></b><b><c>A1</c><d>B1</d></b><b><c>A2</c><d>B2</d></b></a>");

x1.b[0] = new XML("<b><c>A3</c><d>B3</d></b>");

y1 = new XML("<a><b><c>A3</c><d>B3</d></b><b><c>A1</c><d>B1</d></b><b><c>A2</c><d>B2</d></b></a>");

Assert.expectEq( "AssignmentExpression = XML value        :", true, (x1==y1) );


// AssignmentExpression = XMLList object

x1 = new XML("<a><b><c>A0</c><d>B0</d></b><b><c>A1</c><d>B1</d></b><b><c>A2</c><d>B2</d></b></a>");

x1.b = new XMLList("<b>A</b><b>B</b><b>C</b>");

y1 = new XML("<a><b>A</b><b>B</b><b>C</b></a>");

Assert.expectEq( "AssignmentExpression = XMLList object   :", true, (x1==y1) );


// AssignmentExpression != XML/XMLList

x1 = new XML("<a><b><c>A0</c><d>B0</d></b><b><c>A1</c><d>B1</d></b><b><c>A2</c><d>B2</d></b></a>");

x1.b = "Hello World";

y1 = new XML("<a><b>Hello World</b></a>");

Assert.expectEq( "AssignmentExpression != XML/XMLList     :", true, (x1==y1) );

END();
