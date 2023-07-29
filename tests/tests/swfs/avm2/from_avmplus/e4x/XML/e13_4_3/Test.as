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

START("13.4.3 - XML Properties");

// Test defaults
TEST(1, true, XML.ignoreComments);
TEST(2, true, XML.ignoreProcessingInstructions);
TEST(3, true, XML.ignoreWhitespace);
TEST(4, true, XML.prettyPrinting);
TEST(5, 2, XML.prettyIndent);

// ignoreComments
x1 = <alpha><!-- comment --><bravo>one</bravo></alpha>;

correct = <alpha><bravo>one</bravo></alpha>;

TEST(6, correct, x1);

XML.ignoreComments = false;

x1 = <alpha><!-- comment --><bravo>one</bravo></alpha>;

correct =
"<alpha>" + NL() +
"  <!-- comment -->" + NL() +
"  <bravo>one</bravo>" + NL() +
"</alpha>";

TEST_XML(7, correct, x1);


// ignoreProcessingInstructions
XML.defaultSettings();
x1 =
<>
    <alpha>
        <?foo version="1.0" encoding="utf-8"?>
        <bravo>one</bravo>
    </alpha>
</>;

correct =
<alpha>
    <bravo>one</bravo>
</alpha>;

TEST(8, correct, x1);

XML.ignoreProcessingInstructions = false;

x1 =
<>
    <alpha>
        <?foo version="1.0" encoding="utf-8"?>
        <bravo>one</bravo>
    </alpha>
</>;

correct =
"<alpha>" + NL() +
"  <?foo version=\"1.0\" encoding=\"utf-8\"?>" + NL() +
"  <bravo>one</bravo>" + NL() +
"</alpha>";

TEST_XML(9, correct, x1);

// ignoreWhitespace
XML.defaultSettings();
x1 = new XML("<alpha> \t\r\n\r\n<bravo> \t\r\n\r\none</bravo> \t\r\n\r\n</alpha>");

correct =
"<alpha>" + NL() +
"  <bravo>one</bravo>" + NL() +
"</alpha>";

TEST_XML(10, correct, x1);

XML.ignoreWhitespace = false;
XML.prettyPrinting = false;

correct = "<alpha> \n\n<bravo> \n\none</bravo> \n\n</alpha>";
x1 = new XML(correct);

TEST_XML(11, correct, x1);

// prettyPrinting
XML.prettyPrinting = true;

x1 =<alpha>one<bravo>two</bravo><charlie/><delta>three<echo>four</echo></delta></alpha>;

correct = "<alpha>" + NL() +
    "  one" + NL() +
    "  <bravo>two</bravo>" + NL() +
    "  <charlie/>" + NL() +
    "  <delta>" + NL() +
    "    three" + NL() +
    "    <echo>four</echo>" + NL() +
    "  </delta>" + NL() +
    "</alpha>";
    
TEST(12, correct, x1.toString());
TEST(13, correct, x1.toXMLString());

XML.prettyPrinting = false;

correct = "<alpha>one<bravo>two</bravo><charlie/><delta>three<echo>four</echo></delta></alpha>";
TEST(14, correct, x1.toString());
TEST(15, correct, x1.toXMLString());

// prettyIndent
XML.prettyPrinting = true;
XML.prettyIndent = 3;

correct = "<alpha>" + NL() +
    "   one" + NL() +
    "   <bravo>two</bravo>" + NL() +
    "   <charlie/>" + NL() +
    "   <delta>" + NL() +
    "      three" + NL() +
    "      <echo>four</echo>" + NL() +
    "   </delta>" + NL() +
    "</alpha>";

TEST(16, correct, x1.toString());
TEST(17, correct, x1.toXMLString());

XML.prettyIndent = 0;

correct = "<alpha>" + NL() +
    "one" + NL() +
    "<bravo>two</bravo>" + NL() +
    "<charlie/>" + NL() +
    "<delta>" + NL() +
    "three" + NL() +
    "<echo>four</echo>" + NL() +
    "</delta>" + NL() +
    "</alpha>";

TEST(18, correct, x1.toString());
TEST(19, correct, x1.toXMLString());

// settings()
XML.defaultSettings();
o = XML.settings();
TEST(20, false, o.ignoreComments);
TEST(21, false, o.ignoreProcessingInstructions);
TEST(22, false, o.ignoreWhitespace);
TEST(23, true, o.prettyPrinting);
TEST(24, 0, o.prettyIndent);

// setSettings()
o = XML.settings();
o.ignoreComments = false;
o.ignoreProcessingInstructions = false;
o.ignoreWhitespace = false;
o.prettyPrinting = false;
o.prettyIndent = 7;

XML.setSettings(o);
o = XML.settings();
TEST(25, false, o.ignoreComments);
TEST(26, false, o.ignoreProcessingInstructions);
TEST(27, false, o.ignoreWhitespace);
TEST(28, false, o.prettyPrinting);
TEST(29, 7, o.prettyIndent);

// defaultSettings()
XML.defaultSettings();
o = XML.settings();
TEST(30, false, o.ignoreComments);
TEST(31, false, o.ignoreProcessingInstructions);
TEST(32, false, o.ignoreWhitespace);
TEST(33, false, o.prettyPrinting);
TEST(34, 7, o.prettyIndent);

END();
