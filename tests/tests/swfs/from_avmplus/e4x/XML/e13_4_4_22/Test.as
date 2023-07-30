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

START("13.4.4.22 - XML name()");

//TEST(1, true, XML.prototype.hasOwnProperty("name"));

x1 =
<alpha>
    <bravo>one</bravo>
    <charlie>
        <bravo>two</bravo>
    </charlie>
</alpha>;

y1 = x1.bravo.name();

TEST(2, "object", typeof(y1));
TEST(3, QName("bravo"), y1);

x1 =
<foo:alpha xmlns:foo="http://foo/">
    <foo:bravo name="one" foo:value="two">one</foo:bravo>
</foo:alpha>;

ns = new Namespace("http://foo/");
y1 = x1.ns::bravo.name();

TEST(4, "object", typeof(y1));
TEST(5, QName("http://foo/", "bravo"), y1);

y1 = x1.ns::bravo.@name.name();
TEST(6, QName("name"), y1);

y1 = x1.ns::bravo.@ns::value.name();
TEST(7, "http://foo/", y1.uri);
TEST(8, "value", y1.localName);
TEST(9, QName("http://foo/", "value"), y1);

function convertToString(o:Object){
  return o.toString();
}

var xmlDoc = "<company xmlns:printer='http://colors.com/printer/'><printer:employee id='1'><name>John</name> <city>California</city> </printer:employee></company>";

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.name()",
    convertToString(new QName("company")),
    (MYXML = new XML(xmlDoc), MYXML.name()).toString());

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.name() instanceof QName",
    true,
    (MYXML = new XML(xmlDoc), MYXML.name() instanceof QName));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.name().toString()",
    "company",
    (MYXML = new XML(xmlDoc), MYXML.name().toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.children()[0].name()",

    convertToString(new QName("http://colors.com/printer/", "employee")),

    (MYXML = new XML(xmlDoc), MYXML.children()[0].name()).toString()
    );

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.children()[0].name() instanceof QName",
    true,
    (MYXML = new XML(xmlDoc), MYXML.children()[0].name() instanceof QName));
    
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.children()[0].name().toString()",
    "http://colors.com/printer/::employee",
    (MYXML = new XML(xmlDoc), MYXML.children()[0].name().toString()));

END();
