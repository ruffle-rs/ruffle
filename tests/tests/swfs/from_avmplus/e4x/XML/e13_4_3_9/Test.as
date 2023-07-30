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

START("13.4.3.9 - XML.defaultSettings()");

Assert.expectEq( "settings = XML.defaultSettings(), settings.ignoreComments", true,
             (settings = XML.defaultSettings(), settings.ignoreComments) );
Assert.expectEq( "settings = XML.defaultSettings(), settings.ignoreProcessingInstructions", true,
             (settings = XML.defaultSettings(), settings.ignoreProcessingInstructions) );
Assert.expectEq( "settings = XML.defaultSettings(), settings.ignoreWhitespace", true,
             (settings = XML.defaultSettings(), settings.ignoreWhitespace) );
Assert.expectEq( "settings = XML.defaultSettings(), settings.prettyPrinting", true,
             (settings = XML.defaultSettings(), settings.prettyPrinting) );
Assert.expectEq( "settings = XML.defaultSettings(), settings.prettyIndent", 2,
             (settings = XML.defaultSettings(), settings.prettyIndent) );

var tempSettings = XML.settings();
tempSettings.ignoreComments = false;
tempSettings.ignoreProcessingInstructions = false;
tempSettings.ignoreWhitespace = false;
tempSettings.prettyPrinting = false;
tempSettings.prettyIndent = 4;

Assert.expectEq( "XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.ignoreComments", true,
             (XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.ignoreComments) );
Assert.expectEq( "XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.ignoreProcessingInstructions", true,
             (XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.ignoreProcessingInstructions) );
Assert.expectEq( "XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.ignoreWhitespace", true,
             (XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.ignoreWhitespace) );
Assert.expectEq( "XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.prettyPrinting", true,
             (XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.prettyPrinting) );
Assert.expectEq( "XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.prettyIndent", 2,
             (XML.setSettings(tempSettings), XML.setSettings(XML.defaultSettings()), XML.prettyIndent) );

END();
