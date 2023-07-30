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

START("13.5.4.9 - XMLList descendants()");

//TEST(1, true, XMLList.prototype.hasOwnProperty("descendants"));
    
XML.prettyPrinting = false;

var xmlDoc = "<MLB><foo>bar</foo><Team>Giants<foo>bar</foo></Team><City><foo>bar</foo>San Francisco</City><Team>Padres</Team><City>San Diego</City></MLB>";

Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('Team')", "<Team>Giants<foo>bar</foo></Team>" + NL() + "<Team>Padres</Team>",
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('Team').toString()) );
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('Team').length()", 2,
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('Team').length()) );
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('Team') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('Team') instanceof XMLList) );

// find multiple levels of descendents
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('foo')", "<foo>bar</foo>" + NL() + "<foo>bar</foo>" + NL() + "<foo>bar</foo>",
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('foo').toString()) );
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('foo').length()", 3,
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('foo').length()) );
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('foo') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('foo') instanceof XMLList) );

// no matching descendents
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('nomatch')", "",
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('nomatch').toString()) );
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('nomatch').length()", 0,
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('nomatch').length()) );
Assert.expectEq( "MYXML = new XMLList(xmlDoc), MYXML.descendants('nomatch') instanceof XMLList", true,
             (MYXML = new XMLList(xmlDoc), MYXML.descendants('nomatch') instanceof XMLList) );

END();
