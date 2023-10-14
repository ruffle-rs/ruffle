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



function writeTestCaseResult(d,e,a)
{
  trace("writeTestCaseResult: " + d + ", expected " + e + ", got " + a );
}


function test(... rest:Array) {

    if( rest.length == 0 ){
        // no args sent, use default test
        for ( var tc=0; tc < testcases.length; tc++ ) {
            testcases[tc].passed = writeTestCaseResult(
                    testcases[tc].expect,
                    testcases[tc].actual,
                    testcases[tc].description +" = "+ testcases[tc].actual );
            testcases[tc].reason += checkReason(testcases[tc].passed);
        }
    } else {
        // we need to use a specialized call to writeTestCaseResult
        if( rest[0] == "no actual" ){
            for ( var tc=0; tc < testcases.length; tc++ ) {
                testcases[tc].passed = writeTestCaseResult(
                                    testcases[tc].expect,
                                    testcases[tc].actual,
                                    testcases[tc].description );
                testcases[tc].reason += checkReason(testcases[tc].passed);
        }
        }
    }
   // stopTest();
    return ( testcases );
}


START("13.4.4.28 - processingInsructions()");

//TEST(1, true, XML.prototype.hasOwnProperty("processingInstructions"));

XML.ignoreProcessingInstructions = false;

// test generic PI
var x1 = new XML("<alpha><?xyz abc=\"123\" michael=\"weird\"?><?another name=\"value\" ?><bravo>one</bravo></alpha>");

var correct = new XMLList("<?xyz abc=\"123\" michael=\"weird\"?><?another name=\"value\" ?>");

TEST(2, correct, x1.processingInstructions());
TEST(3, correct, x1.processingInstructions("*"));

correct = "<?xyz abc=\"123\" michael=\"weird\"?>";

TEST_XML(4, correct, x1.processingInstructions("xyz"));

// test XML-decl
// Un-comment these tests when we can read in doc starting with PI.
x1 = new XML("<?xml version=\"1.0\" ?><alpha><bravo>one</bravo></alpha>");

correct = new XMLList("<?xml version=\"1.0\" encoding=\"utf-8\"?>");

test(5, correct, x1.processingInstructions());
test(6, correct, x1.processingInstructions("*"));
test(7, correct, x1.processingInstructions("xml"));

// extra whitespace is on purpose for <?foo              bar> at the end of this string
var xmlDoc = "<?xml version='1.0'?><xml><?xml-stylesheet href='mystyle.xsl'?><employee id='1'><firstname>John</firstname><lastname>Walton</lastname><age>25</age></employee> <employee id='2'><firstname>Sue</firstname><lastname>Day</lastname><age>32</age><?child-xml bar?></employee><?foo              bar?></xml>"

// propertyName as a string
XML.ignoreProcessingInstructions = true;

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions().toString()", "",
             (MYXML = new XML(xmlDoc), MYXML.processingInstructions().toString()));

XML.ignoreProcessingInstructions = false;

Assert.expectEq( "ignorePI = false, MYXML = new XML(xmlDoc), MYXML.processingInstructions().toString()",
             "<?xml-stylesheet href='mystyle.xsl'?>\n<?foo bar?>",
             (MYXML = new XML(xmlDoc), MYXML.processingInstructions().toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions('*')",
    "<?xml-stylesheet href='mystyle.xsl'?>\n<?foo bar?>",
    (MYXML = new XML(xmlDoc), MYXML.processingInstructions("*").toXMLString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions('xml-stylesheet')",
    "<?xml-stylesheet href='mystyle.xsl'?>",
    (MYXML = new XML(xmlDoc), MYXML.processingInstructions("xml-stylesheet").toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions(new QName('xml-stylesheet'))",
    "<?xml-stylesheet href='mystyle.xsl'?>",
    (MYXML = new XML(xmlDoc), MYXML.processingInstructions(new QName("xml-stylesheet")).toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions(new QName('foo'))",
    "<?foo bar?>",
    (MYXML = new XML(xmlDoc), MYXML.processingInstructions(new QName("foo")).toString()));

// Attribute name does not match
Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions('@xml-stylesheet')",
    "",
    (MYXML = new XML(xmlDoc), MYXML.processingInstructions("@xml-stylesheet").toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions('xml-foo')",
    "",
    (MYXML = new XML(xmlDoc), MYXML.processingInstructions("xml-foo").toString()));

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.processingInstructions('child-xml')",
    "",
    (MYXML = new XML(xmlDoc), MYXML.processingInstructions("child-xml").toString()));

END();
