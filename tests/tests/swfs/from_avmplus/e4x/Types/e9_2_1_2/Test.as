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

START("9.2.1.2 - XMLList [[Put]]");


// element

var x1 = new XMLList("<alpha>one</alpha><bravo>two</bravo>");

TEST(1, "<alpha>one</alpha>" + NL() + "<bravo>two</bravo>",
  x1.toXMLString());

x1[0] = <charlie>three</charlie>;
TEST(2, "<charlie>three</charlie>" + NL() + "<bravo>two</bravo>",
  x1.toXMLString());

x1[0] = <delta>four</delta> + <echo>five</echo>;
TEST(3, "<delta>four</delta>" + NL() + "<echo>five</echo>" + NL() + "<bravo>two</bravo>",
  x1.toXMLString());
  
var y1 = new XMLList("<alpha>one</alpha><bravo>two</bravo>");
y1[0] = "five";

TEST(4, "<alpha>five</alpha>" + NL() + "<bravo>two</bravo>",
  y1.toXMLString());
  


// attribute

var x1 = new XMLList("<alpha attr=\"50\">one</alpha><bravo>two</bravo>");
x1[0].@attr = "fifty";
TEST(5, "<alpha attr=\"fifty\">one</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());

var x1 = new XMLList("<alpha attr=\"50\">one</alpha><bravo>two</bravo>");
x1[0].@attr = new XMLList("<att>sixty</att>");
TEST(6, "<alpha attr=\"sixty\">one</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());

var x1 = new XMLList("<alpha attr=\"50\">one</alpha><bravo>two</bravo>");
x1[0].@attr = "<att>sixty</att>";
TEST(7, "<alpha attr=\"&lt;att>sixty&lt;/att>\">one</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());


// text node

var x1 = new XMLList("alpha<bravo>two</bravo>");
x1[0] = "beta";
TEST(8, "beta" + NL() + "<bravo>two</bravo>", x1.toXMLString());

var x1 = new XMLList("<alpha>one</alpha><bravo>two</bravo>");
x1[0] = new XML("beta");
TEST(9, "<alpha>beta</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());

var x1 = new XMLList("<alpha>one</alpha><bravo>two</bravo>");
x1[0] = new XMLList("beta");
TEST(10, "<alpha>beta</alpha>" + NL() + "<bravo>two</bravo>", x1.toXMLString());

var x1 = new XMLList("alpha<bravo>two</bravo>");
x1[0] = new XML("<two>beta</two>");
TEST(11, "<two>beta</two>" + NL() + "<bravo>two</bravo>", x1.toXMLString());


// comment

var x1 = new XMLList("<alpha><beta><!-- hello, comment --></beta></alpha>");
x1.beta = new XMLList("<comment>hello</comment>");
TEST(12, new XMLList("<alpha><comment>hello</comment></alpha>"), x1);

var x1 = new XMLList("<alpha><beta><!-- hello, comment --></beta></alpha>");
x1.beta = new XML("<comment>hello</comment>");
TEST(13, new XMLList("<alpha><comment>hello</comment></alpha>"), x1);

var x1 = new XMLList("<alpha><beta><!-- hello, comment --></beta></alpha>");
x1.beta = "hello";
TEST(14, new XMLList("<alpha><beta>hello</beta></alpha>"), x1);

var x1 = new XMLList("<alpha><beta><!-- hello, comment --></beta></alpha>");
x1.beta = new XML("hello");
TEST(15, new XMLList("<alpha><beta>hello</beta></alpha>"), x1);


// PI

XML.ignoreProcessingInstructions = false;
var x1 = new XML("<alpha><beta><?xm-xsl-param name=\"sponsor\" value=\"dw\"?></beta></alpha>");
x1.beta = new XML("<pi element=\"yes\">instructions</pi>");
TEST(16, new XMLList("<alpha><pi element=\"yes\">instructions</pi></alpha>"), x1);

XML.ignoreProcessingInstructions = false;
var x1 = new XML("<alpha><beta><?xm-xsl-param name=\"sponsor\" value=\"dw\"?></beta></alpha>");
x1.beta = new XMLList("<pi element=\"yes\">instructions</pi>");
TEST(17, new XMLList("<alpha><pi element=\"yes\">instructions</pi></alpha>"), x1);

var x1 = new XML("<alpha><beta><?xm-xsl-param name=\"sponsor\" value=\"dw\"?></beta></alpha>");
x1.beta = "processing instructions";
TEST(18, new XMLList("<alpha><beta>processing instructions</beta></alpha>"), x1);

END();
