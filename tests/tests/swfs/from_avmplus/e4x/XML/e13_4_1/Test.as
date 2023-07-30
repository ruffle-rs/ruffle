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

START("13.4.1 - XML Constructor as Function");

x1 = XML();
TEST(1, "xml", typeof(x1));
TEST(2, true, x1 instanceof XML);

correct =
<Envelope
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:stock="http://mycompany.com/stocks"
    soap:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
    <Body>
        <stock:GetLastTradePrice>
            <stock:symbol>DIS</stock:symbol>
        </stock:GetLastTradePrice>
    </Body>
</Envelope>;

x1 = XML(correct);
TEST(3, correct, x1);

text =
"<Envelope" +
"    xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\"" +
"    xmlns:stock=\"http://mycompany.com/stocks\"" +
"    soap:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">" +
"    <Body>" +
"        <stock:GetLastTradePrice>" +
"            <stock:symbol>DIS</stock:symbol>" +
"        </stock:GetLastTradePrice>" +
"    </Body>" +
"</Envelope>";

x1 =  XML(text);
TEST(4, correct, x1);

// Make sure it's not copied if it's XML
x1 =
<alpha>
    <bravo>two</bravo>
</alpha>;

y1 = XML(x1);

x1.bravo = "three";

correct =
<alpha>
    <bravo>three</bravo>
</alpha>;

TEST(5, correct, y1);

// Make text node
x1 = XML("4");
TEST_XML(6, 4, x1);

x1 = XML(4);
TEST_XML(7, 4, x1);
 
// Undefined and null should behave like ""
x1 = XML(null);
TEST_XML(8, "", x1);

x1 = XML(undefined);
TEST_XML(9, "", x1);
  
XML.prettyPrinting = false;

var thisXML = "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>";
var NULL_OBJECT = null;
// value is null
Assert.expectEq( "XML(null).valueOf().toString()", "", XML(null).valueOf().toString() );
Assert.expectEq( "typeof XML(null)", "xml", typeof XML(null) );

// value is undefined
Assert.expectEq( "XML(undefined).valueOf().toString()", "", XML(undefined).valueOf().toString() );
Assert.expectEq( "typeof XML(undefined)", "xml", typeof XML(undefined) );

// value is not supplied
Assert.expectEq( "XML().valueOf().toString()", "", XML().valueOf().toString() );
Assert.expectEq( "typeof XML()", "xml", typeof XML() );

// value is supplied
Assert.expectEq( "XML(thisXML).valueOf().toString()", thisXML, XML(thisXML).valueOf().toString() );
Assert.expectEq( "typeof XML(thisXML)", "xml", typeof XML(thisXML) );

END();
