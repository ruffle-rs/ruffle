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

START("13.4.3.3 - XML.ignoreProcessingInstructions");

// We set this to false so we can more easily compare string output
XML.prettyPrinting = false;

// xml string with processing instructions
var xmlDoc = "<?xml version=\"1.0\"?><XML><?xml-stylesheet href=\"classic.xsl\" type=\"text/xml\"?><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>"

// a) value of ignoreProcessingInstructions
Assert.expectEq ("XML.ignoreProcessingInstructions", true, (XML.ignoreProcessingInstructions));
Assert.expectEq( "XML.ignoreProcessingInstructions = false, XML.ignoreProcessingInstructions", false, (XML.ignoreProcessingInstructions = false, XML.ignoreProcessingInstructions));
Assert.expectEq( "XML.ignoreProcessingInstructions = true, XML.ignoreProcessingInstructions", true, (XML.ignoreProcessingInstructions = true, XML.ignoreProcessingInstructions));

// b) if ignoreProcessingInstructions is true, XML processing instructions are ignored when construction the new XML objects
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.toString()", "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>",
             (XML.ignoreProcessingInstructions = true, MYXML = new XML(xmlDoc), MYXML.toString() ));
             
// !!@ note that the "<?xml version=\"1.0\"?>" tag magically disappeared.
             
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.toString() with ignoreProcessingInstructions=false",
        "<XML><?xml-stylesheet href=\"classic.xsl\" type=\"text/xml\"?><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>",
        (XML.ignoreProcessingInstructions = false, MYXML = new XML(xmlDoc), MYXML.toString() ));


// If ignoreProcessingInstructions is true, XML constructor from another XML node ignores processing instructions
XML.ignoreProcessingInstructions = false;
var MYXML = new XML(xmlDoc); // this XML node has processing instructions
XML.ignoreProcessingInstructions = true;
var xml2 = new XML(MYXML); // this XML tree should not have processing instructions
Assert.expectEq( "xml2 = new XML(MYXML), xml2.toString()", "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>",
             (xml2.toString()) );
XML.ignoreProcessingInstructions = false;
var xml3 = new XML(MYXML); // this XML tree will have processing instructions
Assert.expectEq( "xml3 = new XML(MYXML), xml3.toString()",
        "<XML><?xml-stylesheet href=\"classic.xsl\" type=\"text/xml\"?><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>",
             (xml3.toString()) );


END();
