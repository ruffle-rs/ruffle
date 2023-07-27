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

START("11.2.4 - XML Filtering Predicate Operator");

function convertToString(o:Object){
  return o.toString();
}

var p;

e = <employees>
    <employee id="0"><fname>John</fname><age>20</age></employee>
    <employee id="1"><fname>Sue</fname><age>30</age></employee>
    </employees>;


correct = <employee id="0"><fname>John</fname><age>20</age></employee>;

john = e.employee.(fname == "John");
TEST(1, correct, john);

john = e.employee.(fname == "John");
TEST(2, correct, john);

correct =
<employee id="0"><fname>John</fname><age>20</age></employee> +
<employee id="1"><fname>Sue</fname><age>30</age></employee>;

twoEmployees = e.employee.(@id == 0 || @id == 1);
TEST(3, correct, twoEmployees);

twoEmployees = e.employee.(@id == 0 || @id == 1);
TEST(4, correct, twoEmployees);

i = 0;
twoEmployees = new XMLList();
for each (p in e..employee)
{
    if (p.@id == 0 || p.@id == 1)
    {
        twoEmployees += p;
    }
}
TEST(5, correct, twoEmployees);

i = 0;
twoEmployees = new XMLList();
for each (p in e..employee)
{
    if (p.@id == 0 || p.@id == 1)
    {
        twoEmployees[i++] = p;
    }
}
TEST(6, correct, twoEmployees);

// test with syntax
e = <employees>
    <employee id="0"><fname>John</fname><age>20</age></employee>
    <employee id="1"><fname>Sue</fname><age>30</age></employee>
    </employees>;

correct =
<employee id="0"><fname>John</fname><age>20</age></employee> +
<employee id="1"><fname>Sue</fname><age>30</age></employee>;

i = 0;
twoEmployees = new XMLList();
for each (p in e..employee)
{
    with (p)
    {
        if (@id == 0 || @id == 1)
        {
            twoEmployees[i++] = p;
        }
    }
}
TEST(7, correct, twoEmployees);

var xml = "<employees><employee id=\"1\"><fname>Joe</fname><age>20</age></employee><employee id=\"2\"><fname>Sue</fname><age>Joe</age></employee></employees>";
var e = new XML(xml);

// get employee with fname Joe
Assert.expectEq("e.employee.(fname == \"Joe\")", 1, (joe = e.employee.(fname == "Joe"), joe.length()));


// employees with id's 0 & 1
Assert.expectEq("employees with id's 1 & 2", 2, (emps = e.employee.(@id == 1 || @id == 2), emps.length()));


// name of employee with id 1
Assert.expectEq("name of employee with id 1", "Joe", (emp = e.employee.(@id == 1).fname, emp.toString()));


// get the two employees with ids 0 and 1 using a predicate
var i = 0;
var twoEmployees = new XMLList();
for each (p in e..employee) {
    with (p) {
        if (@id == 1 || @id == 2) {
            twoEmployees[i++] = p;
        }
    }
}

var twoEmployees = e..employee.(@id == 1 || @id == 2);

Assert.expectEq("Compare to equivalent XMLList", true, (emps = e..employee.(@id == 1 || @id == 2), emps == twoEmployees));

 var employees:XML =
<employees>
<employee ssn="123-123-1234" id="1">
<name first="John" last="Doe"/>
<address>
<street>11 Main St.</street>
<city>San Francisco</city>
<state>CA</state>
<zip>98765</zip>
</address>
</employee>
<employee ssn="789-789-7890" id="2">
<name first="Mary" last="Roe"/>
<address>
<street>99 Broad St.</street>
<city>Newton</city>
<state>MA</state>
<zip>01234</zip>
</address>
</employee>
</employees>;

for each (var id:XML in employees.employee.@id) {
trace(id); // 123-123-1234
}

correct =
<employee ssn="789-789-7890" id="2">
<name first="Mary" last="Roe"/>
<address>
<street>99 Broad St.</street>
<city>Newton</city>
<state>MA</state>
<zip>01234</zip>
</address>
</employee>;

var idToFind:String = "2";
Assert.expectEq("employees.employee.(@id == idToFind)", correct.toString(), (employees.employee.(@id == idToFind)).toString());


END();
