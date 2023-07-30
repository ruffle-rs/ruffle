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

START("11.2.3 - XML Descendant Accessor");

function convertToString(o:Object){
  return o.toString();
}
var e =
<employees>
    <employee id="1"><name>Joe</name><age>20</age></employee>
    <employee id="2"><name>Sue</name><age>30</age></employee>
</employees>

names = e..name;

correct =
<name>Joe</name> +
<name>Sue</name>;

TEST(1, correct, names);

e = "<employees><employee id=\"1\"><name>Joe</name><age>20</age></employee><employee id=\"2\"><name>Sue</name><age>30</age></employee></employees>";

Assert.expectEq("xml..validnode:", "Joe", (x1 = new XML(e), names = x1..name, names[0].toString()));

Assert.expectEq("xml..validnode length:", 2, (x1 = new XML(e), names = x1..name, names.length()));

Assert.expectEq("xml..invalidnode:", undefined, (x1 = new XML(e), names = x1..hood, names[0]));

Assert.expectEq("xmllist..validnode:", "Joe", (x1 = new XMLList(e), names = x1..name, names[0].toString()));

Assert.expectEq("xmllist..invalidnode:", undefined, (x1 = new XMLList(e), names = x1..hood, names[0]));

Assert.expectEq("xmllist..invalidnode length:", 0, (x1 = new XMLList(e), names = x1..hood, names.length()));

e =
<employees>
    <employee id="1"><first_name>Joe</first_name><age>20</age></employee>
    <employee id="2"><first_name>Sue</first_name><age>30</age></employee>
</employees>

correct =
<first_name>Joe</first_name> +
<first_name>Sue</first_name>;

names = e..first_name;

TEST(2, correct, names);

e =
<employees>
    <employee id="1"><first-name>Joe</first-name><age>20</age></employee>
    <employee id="2"><first-name>Sue</first-name><age>30</age></employee>
</employees>

e =
<company><staff>
    <bug attr='1'><coat><bug>heart</bug></coat></bug>
    <bug attr='2'><dirt><bug>part</bug></dirt></bug>
</staff></company>

es = <><bug attr='1'><coat><bug>heart</bug></coat></bug><bug>heart</bug><bug attr='2'><dirt><bug>part</bug></dirt></bug><bug>part</bug></>;

Assert.expectEq(3, es.toString(), convertToString(e..bug));

END();
