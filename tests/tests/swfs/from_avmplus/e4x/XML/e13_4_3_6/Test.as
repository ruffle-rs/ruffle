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

START("13.4.3.6 - XML.prettyIndent");

// xml doc
XML.prettyPrinting = true;
var xmlDoc = "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY><SPORT>Baseball</SPORT></XML>"


// a) value of prettyIndent
Assert.expectEq( "XML.prettyIndent = 4, XML.prettyIndent", 4, (XML.prettyIndent = 4, XML.prettyIndent));
Assert.expectEq( "XML.prettyIndent = 2, XML.prettyIndent", 2, (XML.prettyIndent = 2, XML.prettyIndent));
Assert.expectEq( "XML.prettyIndent = -1, XML.prettyIndent", -1, (XML.prettyIndent = -1, XML.prettyIndent));

// b) pretty printing

Assert.expectEq( "MYOB = new XML(xmlDoc); XML.prettyIndent = 2; MYOB.toString()",
            "<XML>" + NL() + "  <TEAM>Giants</TEAM>" + NL() + "  <CITY>San Francisco</CITY>" + NL() + "  <SPORT>Baseball</SPORT>" + NL() + "</XML>",
             (MYOB = new XML(xmlDoc), XML.prettyIndent = 2, MYOB.toString()));

Assert.expectEq( "MYOB = new XML(xmlDoc); XML.prettyIdent = 4; MYOB.toString()",
            "<XML>" + NL() + "    <TEAM>Giants</TEAM>" + NL() + "    <CITY>San Francisco</CITY>" + NL() + "    <SPORT>Baseball</SPORT>" + NL() + "</XML>",
             (MYOB = new XML(xmlDoc), XML.prettyIndent = 4, MYOB.toString()));

Assert.expectEq( "MYOB = new XML(xmlDoc); XML.prettyIndent = 1; MYOB.toString()",
            "<XML>" + NL() + " <TEAM>Giants</TEAM>" + NL() + " <CITY>San Francisco</CITY>" + NL() + " <SPORT>Baseball</SPORT>" + NL() + "</XML>",
             (MYOB = new XML(xmlDoc), XML.prettyIndent = 1, MYOB.toString()));

// !!@ bad value causes pretty printing to be disabled
Assert.expectEq( "MYOB = new XML(xmlDoc); XML.prettyIndent = -5; MYOB.toString()",
            "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY><SPORT>Baseball</SPORT></XML>",
            //"<XML>" + NL() + "  <TEAM>Giants</TEAM>" + NL() + "  <CITY>San Francisco</CITY>" + NL() + "  <SPORT>Baseball</SPORT>" + NL() + "</XML>",
             (MYOB = new XML(xmlDoc), XML.prettyIndent = -5, MYOB.toString()));



// !!@ very simple example of printing output
XML.prettyPrinting = true;
XML.prettyIndent = 10;
Assert.expectEq( "MYOB = new XML(xmlDoc); XML.prettyPrinting = true; MYOB.toString()",
            "<XML>" + NL() + "          <TEAM>Giants</TEAM>" + NL() + "          <CITY>San Francisco</CITY>" + NL() + "          <SPORT>Baseball</SPORT>" + NL() + "</XML>",
             (MYOB = new XML(xmlDoc), XML.prettyPrinting = true, MYOB.toString()));


// d) attributes

XML.prettyIndent = 5;
xmlDoc = "<XML><TEAM attr1=\"attr\" attr2=\"hi\">Giants</TEAM><CITY>San Francisco</CITY><SPORT>Baseball</SPORT></XML>"
Assert.expectEq( "MYOB = new XML(xmlDoc); XML.prettyPrinting = true; MYOB.toString()",
            "<XML>" + NL() + "     <TEAM attr1=\"attr\" attr2=\"hi\">Giants</TEAM>" + NL() + "     <CITY>San Francisco</CITY>" + NL() + "     <SPORT>Baseball</SPORT>" + NL() + "</XML>",
             (MYOB = new XML(xmlDoc), XML.prettyPrinting = true, MYOB.toString()));


END();
