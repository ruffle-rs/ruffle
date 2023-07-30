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

START("bug153363_2 - XML.toXMLString");

var myxml = <order xmlns:x='x'>
<item id='1' xmlns:x='x2'>
<menuName xmlns:x='x'>burger</menuName>
<price>3.95</price>
</item>
</order>;

var expected_results1:String = <order xmlns:x='x'>
  <item id='1' xmlns:x='x2'>
    <menuName xmlns:x='x'>burger</menuName>
    <price>3.95</price>
  </item>
</order>

TEST(1,expected_results1,myxml.toXMLString());

var myxml2 = <order>
<item id="1">
<menuName xmlns:x="x" x:foo='10'>burger</menuName>
</item>
<item id="2">
<menuName xmlns:x="x" x:foo='20'>salad</menuName>
</item>
</order>;

var expected_results2:String = <order>
<item id="1">
<menuName xmlns:x="x" x:foo='10'>burger</menuName>
</item>
<item id="2">
<menuName xmlns:x="x" x:foo='20'>salad</menuName>
</item>
</order>;

TEST(2,expected_results2,myxml2.toXMLString());


END();

