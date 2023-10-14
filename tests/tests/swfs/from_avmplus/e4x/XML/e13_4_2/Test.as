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
 

START("13.4.2 - XML Constructor");

x1 = new XML();
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

x1 = new XML(correct);
TEST_XML(3, correct.toXMLString(), x1);

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

x1 = new XML(text);
TEST(4, correct, x1);

// Make sure it's a copy
x1 =
<alpha>
    <bravo>one</bravo>
</alpha>;

y1 = new XML(x1);

x1.bravo.prependChild(<charlie>two</charlie>);

correct =
<alpha>
    <bravo>one</bravo>
</alpha>;

TEST(5, correct, y1);

// Make text node
x1 = new XML("4");
TEST_XML(6, "4", x1);

x1 = new XML(4);
TEST_XML(7, "4", x1);

// Undefined and null should behave like ""
x1 = new XML(null);
TEST_XML(8, "", x1);

x1 = new XML(undefined);
TEST_XML(9, "", x1);

var thisXML = "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>";

// value is null
Assert.expectEq( "typeof new XML(null)", "xml", typeof new XML(null) );
Assert.expectEq( "new XML(null) instanceof XML", true, new XML(null) instanceof XML);
Assert.expectEq( "(new XML(null).nodeKind())", "text", (new XML(null)).nodeKind());
Assert.expectEq( "MYOB = new XML(null); MYOB.toString()", "",
             (MYOB = new XML(null), MYOB.toString(), MYOB.toString()) );

// value is undefined
Assert.expectEq( "typeof new XML(undefined)", "xml", typeof new XML(undefined) );
Assert.expectEq( "new XML(undefined) instanceof XML", true, new XML(undefined) instanceof XML);
Assert.expectEq( "(new XML(undefined).nodeKind())", "text", (new XML(undefined)).nodeKind());
Assert.expectEq( "MYOB = new XML(undefined); MYOB.toString()", "",
             (MYOB = new XML(undefined), MYOB.toString(), MYOB.toString()) );

// value is not supplied
Assert.expectEq( "typeof new XML()", "xml", typeof new XML() );
Assert.expectEq( "new XML() instanceof XML", true, new XML() instanceof XML);
Assert.expectEq( "(new XML().nodeKind())", "text", (new XML()).nodeKind());
Assert.expectEq( "MYOB = new XML(); MYOB.toString()", "",
             (MYOB = new XML(), MYOB.toString(), MYOB.toString()) );

//value is a number
Assert.expectEq( "typeof new XML(5)", "xml", typeof new XML(5) );
Assert.expectEq( "new XML(5) instanceof XML", true, new XML(5) instanceof XML);
Assert.expectEq( "(new XML(5).nodeKind())", "text", (new XML(5)).nodeKind());
Assert.expectEq( "MYOB = new XML(5); MYOB.toString()", "5",
             (MYOB = new XML(5), MYOB.toString(), MYOB.toString()) );

//value is

// value is supplied
XML.prettyPrinting = false;
Assert.expectEq( "typeof new XML(thisXML)", "xml", typeof new XML(thisXML) );
Assert.expectEq( "new XML(thisXML) instanceof XML", true, new XML(thisXML) instanceof XML);
Assert.expectEq( "MYOB = new XML(thisXML); MYOB.toString()", "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>",
             (MYOB = new XML(thisXML), MYOB.toString(), MYOB.toString()) );
             
// Strongly typed XML objects
var MYXML1:XML = new XML(thisXML);
Assert.expectEq("Strongly typed XML object", new XML(thisXML).toString(), MYXML1.toString());

var MYXML2:XML = new XML(<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>);
Assert.expectEq("var MYXML:XML = new XML(<x><a>b</a><c>d</c></x>);", new XML(thisXML).toString(), MYXML2.toString());

var MYXML3:XML = <XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>;
Assert.expectEq("var MYXML:XML = <x><a>b</a><c>d</c></x>;", new XML(thisXML).toString(), MYXML3.toString());

var MYXML4:XML = new XML();
var someXML =  new XML();
var someXML = someXML.toString();
Assert.expectEq("var MYXML:XML = new XML()", someXML, MYXML4.toString());

var MYXML5:XML = new XML(5);
Assert.expectEq("var MYXML:XML = new XML(5)", "5", MYXML5.toString());

var a = new XML("<?xml version='1.0' encoding='UTF-8'?><?xml-stylesheet href='mystyle.css' type='text/css'?> <rdf>compiled</rdf>");

Assert.expectEq("XML with PI and comments", "compiled", a.toString());

try {
    var b = new XML("<a/><b/>");
    result = b;
} catch(e1) {
    result = Utils.typeError(e1.toString());
}

Assert.expectEq("new XML(\"<a/><b/>\")", "TypeError: Error #1088", result);

END();
