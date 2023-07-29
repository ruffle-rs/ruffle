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

START("13.4.3.8 - XML.setSettings(settings)");

// a) called with a settings object
var settings = XML.settings();

settings.ignoreComments = false;
settings.ignoreProcessingInstructions = false;
settings.ignoreWhitespace = false;
settings.prettyPrinting = false;
settings.prettyIndent = 4;

Assert.expectEq( "XML.settings().ignoreComments; ", true, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.settings().ignoreProcessingInstructions; ", true, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.settings().ignoreWhitespace; ", true, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.settings().prettyPrinting; ", true, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.settings().prettyIndent; ", 2, (XML.settings().prettyIndent) );

XML.setSettings (settings);

Assert.expectEq( "XML.settings().ignoreComments; ", false, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.settings().ignoreProcessingInstructions; ", false, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.settings().ignoreWhitespace; ", false, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.settings().prettyPrinting; ", false, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.settings().prettyIndent; ", 4, (XML.settings().prettyIndent) );

// setting null restores defaults
XML.setSettings (null);

Assert.expectEq( "XML.settings(null); XML.settings().ignoreComments; ", true, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.settings(null); XML.settings().ignoreProcessingInstructions; ", true, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.settings(null); XML.settings().ignoreWhitespace; ", true, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.settings(null); XML.settings().prettyPrinting; ", true, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.settings(null); XML.settings().prettyIndent; ", 2, (XML.settings().prettyIndent) );

XML.setSettings (settings);
// does setting a bogus value restore defaults? No.
XML.setSettings ([1, 2, 3, 4]);

Assert.expectEq( "XML.settings().ignoreComments; ", false, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.settings().ignoreProcessingInstructions; ", false, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.settings().ignoreWhitespace; ", false, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.settings().prettyPrinting; ", false, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.settings().prettyIndent; ", 4, (XML.settings().prettyIndent) );

// does setting a bogus value restore defaults? No.
XML.setSettings (5);

Assert.expectEq( "XML.settings().ignoreComments; ", false, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.settings().ignoreProcessingInstructions; ", false, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.settings().ignoreWhitespace; ", false, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.settings().prettyPrinting; ", false, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.settings().prettyIndent; ", 4, (XML.settings().prettyIndent) );

// does setting a bogus value restore defaults? No.
XML.setSettings ("");

Assert.expectEq( "XML.setSettings (\"\"); XML.settings().ignoreComments; ", false, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.setSettings (\"\"); XML.settings().ignoreProcessingInstructions; ", false, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.setSettings (\"\"); XML.settings().ignoreWhitespace; ", false, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.setSettings (\"\"); XML.settings().prettyPrinting; ", false, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.setSettings (\"\"); XML.settings().prettyIndent; ", 4, (XML.settings().prettyIndent) );

// this restores defaults
XML.setSettings (undefined);

Assert.expectEq( "XML.setSettings (undefined); XML.settings().ignoreComments; ", true, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.setSettings (undefined); XML.settings().ignoreProcessingInstructions; ", true, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.setSettings (undefined); XML.settings().ignoreWhitespace; ", true, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.setSettings (undefined); XML.settings().prettyPrinting; ", true, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.setSettings (undefined); XML.settings().prettyIndent; ", 2, (XML.settings().prettyIndent) );

// this restore defaults
XML.setSettings (settings);
XML.setSettings ();

Assert.expectEq( "XML.setSettings (); XML.settings().ignoreComments; ", true, (XML.settings().ignoreComments) );
Assert.expectEq( "XML.setSettings (); XML.settings().ignoreProcessingInstructions; ", true, (XML.settings().ignoreProcessingInstructions) );
Assert.expectEq( "XML.setSettings (); XML.settings().ignoreWhitespace; ", true, (XML.settings().ignoreWhitespace) );
Assert.expectEq( "XML.setSettings (); XML.settings().prettyPrinting; ", true, (XML.settings().prettyPrinting) );
Assert.expectEq( "XML.setSettings (); XML.settings().prettyIndent; ", 2, (XML.settings().prettyIndent) );

END();
