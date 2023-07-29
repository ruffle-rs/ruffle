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

START("10.2.1.2 - EscapeAttributeValue(s)");

Assert.expectEq( "EscapeElementValue('\"')       :", "<x attr=\"I said &quot;hi&quot;\">hi</x>", (x1 = <x attr='I said "hi"'>hi</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('<')        :", "<x attr=\"4 &lt; 5\">b</x>", (x1 = <x attr='4 &lt; 5'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('>')        :", "<x attr=\"10 > 9\">b</x>", (x1 = <x attr='10 &gt; 9'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('&')        :", "<x attr=\"Tom &amp; Jerry\">b</x>", (x1 = <x attr='Tom &amp; Jerry'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('&#x9')        :", "<x attr=\"&#x9;\">b</x>", (x1 = <x attr='&#x9;'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('&#xA')        :", "<x attr=\"&#xA;\">b</x>", (x1 = <x attr='&#xA;'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('&#xD')        :", "<x attr=\"&#xD;\">b</x>", (x1 = <x attr='&#xD;'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('\u0009')        :", "<x attr=\"&#x9;\">b</x>", (x1 = <x attr='\u0009'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('\u000A')        :", "<x attr=\"&#xA;\">b</x>", (x1 = <x attr='\u000A'>b</x>, x1.toXMLString()) );
Assert.expectEq( "EscapeElementValue('\u000D')        :", "<x attr=\"&#xD;\">b</x>", (x1 = <x attr='\u000D'>b</x>, x1.toXMLString()) );

END();
