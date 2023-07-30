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

START("13.4.3.4 - XML.ignoreWhitespace");

// We set this to false so we can more easily compare string output
XML.prettyPrinting = false;

// xml doc with white space
var xmlDoc = "<XML>  <TEAM>Giants</TEAM>\u000D<CITY>San\u0020Francisco</CITY>\u000A<SPORT>Baseball</SPORT>\u0009</XML>"


// a) value of ignoreWhitespace
Assert.expectEq( "XML.ignoreWhitespace = false, XML.ignoreWhitespace", false, (XML.ignoreWhitespace = false, XML.ignoreWhitespace));
Assert.expectEq( "XML.ignoreWhitespace = true, XML.ignoreWhitespace", true, (XML.ignoreWhitespace = true, XML.ignoreWhitespace));


// b) whitespace is ignored when true, not ignored when false
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.toString()", "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY><SPORT>Baseball</SPORT></XML>",
             (XML.ignoreWhitespace = true, MYXML = new XML(xmlDoc), MYXML.toString() ));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.toString() with ignoreWhitespace=false",
        "<XML>  <TEAM>Giants</TEAM>\u000D<CITY>San\u0020Francisco</CITY>\u000A<SPORT>Baseball</SPORT>\u0009</XML>",
        (XML.ignoreWhitespace = false, MYXML = new XML(xmlDoc), MYXML.toString() ));

// c) whitespace characters
// tab
xmlDoc = "<a>\t<b>tab</b></a>";
Assert.expectEq( "XML with tab and XML.ignoreWhiteSpace = true", "<a><b>tab</b></a>",
           (XML.ignoreWhitespace = true, MYXML = new XML(xmlDoc), MYXML.toString() ));

Assert.expectEq( "XML with tab and XML.ignoreWhiteSpace = false", "<a>\t<b>tab</b></a>",
           (XML.ignoreWhitespace = false, MYXML = new XML(xmlDoc), MYXML.toString() ));

// new line
xmlDoc = "<a>\n<b>\n\ntab</b>\n</a>";
Assert.expectEq( "XML with new line and XML.ignoreWhiteSpace = true", "<a><b>tab</b></a>",
           (XML.ignoreWhitespace = true, MYXML = new XML(xmlDoc), MYXML.toString() ));

xmlDoc = "<a>\r\n<b>tab</b>\r\n</a>";
Assert.expectEq( "XML with new line and XML.ignoreWhiteSpace = false", "<a>\r\n<b>tab</b>\r\n</a>",
           (XML.ignoreWhitespace = false, MYXML = new XML(xmlDoc), MYXML.toString() ));

// d) attributes

xmlDoc = "<a><b      attr=\"1      2\">tab</b></a>";
Assert.expectEq( "new XML(\"<a><a><b      attr='1      2'>tab</b></a>\")", "<a><b attr=\"1      2\">tab</b></a>",
           (XML.ignoreWhitespace = true, MYXML = new XML(xmlDoc), MYXML.toString() ));
           
Assert.expectEq( "new XML(\"<a><a><b      attr='1      2'>tab</b></a>\")", "<a><b attr=\"1      2\">tab</b></a>",
           (XML.ignoreWhitespace = false, MYXML = new XML(xmlDoc), MYXML.toString() ));


END();
