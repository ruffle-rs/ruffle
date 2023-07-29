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
 
 /*
*
*
* See http://bugzilla.mozilla.org/show_bug.cgi?id=564468
*
*/
//-----------------------------------------------------------------------------

// var SECTION = "564468";
// var VERSION = "";
// var TITLE   = " XMLParser need to use caseless compares for ?XML and CDATA tags";
// var bug = "564468";



// Unable to access xml declaration node via AS code so instead cause an
// XMLParser::kUnterminatedXMLDeclaration error to be thrown by XMLParser.getNext().
// Prior to this fix this code would throw an XMLParser::kUnterminatedProcessingInstruction
// error was thrown since it fell into the "<?" processing instruction check block
var err:String = "no error";
try {
    var y:String = "<?xml version='1.0'?";
    var z:XML = new XML(y);
} catch (e:Error) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("lowercase xml", "Error #1092", err );
}

err = "no error";
try {
    var y:String = "<?XML version='1.0'?";
    var z:XML = new XML(y);
} catch (e:Error) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("uppercase xml", "Error #1092", err );
}

err = "no error";
try {
    var y:String = "<?Xml version='1.0'?";
    var z:XML = new XML(y);
} catch (e:Error) {
    err = Utils.grabError(e, e.toString());
} finally {
    Assert.expectEq("mixed xml", "Error #1092", err );
}


var lowerdata:String = "<text><![cdata[ This <> is some cdata!]]></text>";
var lowerCDATA:XML = new XML(lowerdata);
Assert.expectEq("lower CDATA", "<text><![CDATA[ This <> is some cdata!]]></text>", lowerCDATA.toXMLString() );

var mixeddata:String = "<text><![Cdata[ This <> is some cdata!]]></text>";
var mixedCDATA:XML = new XML(mixeddata);
Assert.expectEq("mixed CDATA", "<text><![CDATA[ This <> is some cdata!]]></text>", mixedCDATA.toXMLString() );

var upperdata:String = "<text><![CDATA[ This <> is some cdata!]]></text>";
var upperCDATA:XML = new XML(upperdata);
Assert.expectEq("upper CDATA", "<text><![CDATA[ This <> is some cdata!]]></text>", upperCDATA.toXMLString() );

