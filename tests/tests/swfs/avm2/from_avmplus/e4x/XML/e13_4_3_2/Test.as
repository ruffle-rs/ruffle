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

START("13.4.3.2 - XML.ignoreComments");

var thisXML = "<XML><!--comment1--><TEAM>Giants</TEAM><CITY>San Francisco</CITY><!--comment2--></XML>"

XML.prettyPrinting = false;

// a) initial value of ignoreComments is true

Assert.expectEq( "XML.ignoreComments", true, XML.ignoreComments);

// toggling value works ok
Assert.expectEq( "XML.ignoreComments = false, XML.ignoreComments", false, (XML.ignoreComments = false, XML.ignoreComments));
Assert.expectEq( "XML.ignoreComments = true, XML.ignoreComments", true, (XML.ignoreComments = true, XML.ignoreComments));

// b) if ignoreComments is true, XML comments are ignored when construction the new XML objects
Assert.expectEq( "MYXML = new XML(thisXML), MYXML.toString()", "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>",
             (XML.ignoreComments = true, MYXML = new XML(thisXML), MYXML.toString() ));
Assert.expectEq( "MYXML = new XML(thisXML), MYXML.toString() with ignoreComemnts=false", "<XML><!--comment1--><TEAM>Giants</TEAM><CITY>San Francisco</CITY><!--comment2--></XML>",
             (XML.ignoreComments = false, MYXML = new XML(thisXML), MYXML.toString() ));

// If ignoreComments is true, XML constructor from another XML node ignores comments
XML.ignoreComments = false;
var MYXML = new XML(thisXML); // this XML node has comments
XML.ignoreComments = true;
var xml2 = new XML(MYXML); // this XML tree should not have comments
Assert.expectEq( "xml2 = new XML(MYXML), xml2.toString()", "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>",
             (xml2.toString()) );
XML.ignoreComments = false;
var xml3 = new XML(MYXML); // this XML tree will have comments
Assert.expectEq( "xml3 = new XML(MYXML), xml3.toString()", "<XML><!--comment1--><TEAM>Giants</TEAM><CITY>San Francisco</CITY><!--comment2--></XML>",
             (xml3.toString()) );


// c) two attributes { DontEnum, DontDelete }
// !!@


END();
