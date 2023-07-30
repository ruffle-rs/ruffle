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
 

START('9.1.1.2 - XML [[Put]]');


// .
var x1 =
<alpha attr1="value1" attr2="value2">
    <bravo>
        one
        <charlie>two</charlie>
    </bravo>
</alpha>;

var correct =
<charlie>new</charlie>;

x1.bravo.charlie = "new"
TEST(1, correct, x1.bravo.charlie)
x1.bravo = <delta>three</delta>
TEST(2, "three", x1.delta.toString())

// .@
x1 = <alpha attr1="value1" attr2="value2"><bravo>one<charlie>two</charlie></bravo></alpha>
x1.@attr1 = "newValue"
TEST_XML(3, "newValue", x1.@attr1)

x1 = <alpha attr1="value1" name="value2"><bravo>one<charlie>two</charlie></bravo></alpha>
x1.@name = "foo";
TEST_XML(4, "foo", x1.@name)

var a = <a><b><c/></b></a>;

try {
    a.b[0] = a;
    result = a;
} catch (e1) {
    result = Utils.typeError(e1.toString());
}


// This might fail in tomorrow's build. 11/02/05
a = <a><b/>some junk<c/>some junk<d/>some junk</a>;
correct = <a><b/>some junk<c/>some junk<d/>some junk<newnode>with some text</newnode></a>;

Assert.expectEq("blah", correct.toString(), (a.*::foo = <newnode>with some text</newnode>, a).toString());


END();
