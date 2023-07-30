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

START("QName in nested functions");

// var bug = 121219;
var actual = "";
var expect = "no error";

function doFrame() {
    function nestedFunc(a, b, c) {
    }
    
    x1 =
    <alpha>
    <bravo attr1="value1" ns:attr1="value3" xmlns:ns="http://someuri">
    <charlie attr1="value2" ns:attr1="value4"/>
    </bravo>
    </alpha>

    var n = new Namespace("http://someuri");

    q = new QName(n, "attr1");
    nestedFunc(7, "value3value4", x1..@[q]);

    var xml1 = "<c><color c='1'>pink</color><color c='2'>purple</color><color c='3'>orange</color></c>";
    var placeHolder = "c";

    nestedFunc("x1.node1[i].@attr", "1",( x2 = new XML(xml1), x2.color[0].@c.toString()));
}

try {
    doFrame();
    actual = "no error";
} catch(e1) {
    actual = "error thrown: " + e1.toString();
}
    


Assert.expectEq("QName in nested function", expect, actual);

END();
