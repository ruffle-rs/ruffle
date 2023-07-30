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

START("11.1.5 - XMLList Initializer");

XML.ignoreWhitespace = true;

var docfrag = <><fName>Phil</fName><age>35</age><hobby>skiing</hobby></>;
TEST(1, "xml", typeof(docfrag));

var correct = <fName>Phil</fName>;
TEST(2, correct, docfrag[0]);

var emplist = <>
          <employee id="0"><name>Jim</name><age>25</age></employee>
          <employee id="1"><name>Joe</name><age>20</age></employee>
          <employee id="2"><name>Sue</name><age>30</age></employee>
          </>;
          
TEST(3, "xml", typeof(emplist));
TEST_XML(4, 2, emplist[2].@id);

var myVar = "kitty";

var xml1 = <><a><b><c>1</c></b></a></>;
var xml2 = <>a</>;
var xml3 = <><a>b</a></>;
var xml4 = <><a b='1'><c>cat</c><d>walk</d></a><a b='2'><c>dog</c><d>run</d></a></>;
var xml5 = <></>;
var xml6 = <><{myVar}>hello</{myVar}></>;
var xml7 = <><c q='1'>"quotes"</c><c q='2'>&#x7B;curly brackets&#x7D;</c></>;
var xml8 = <><a>5 &gt; 4</a></>;
var empxml = <>
    <employee id="0" ><name>Jim</name><age>25</age></employee>
    <employee id="1" ><name>Joe</name><age>20</age></employee>
    <employee id="2" ><name>Sue</name><age>30</age></employee>
</>;
emplist = "<employee id=\"0\" ><name>Jim</name><age>25</age></employee><employee id=\"1\" ><name>Joe</name><age>20</age></employee><employee id=\"2\" ><name>Sue</name><age>30</age></employee>";


Assert.expectEq( "<><a><b><c>1</c></b></a></> == new XMLList(\"<a><b><c>1</c></b></a>\")", true,
           (  x1 = new XMLList('<a><b><c>1</c></b></a>'), (xml1 == x1)));
           
Assert.expectEq( "<>a</> == new XMLList('a')", true,
           (  x1 = new XMLList('a'), (xml2 == x1)));

Assert.expectEq( "<><a>b</a></> == new XMLList('<a>b</a>')", true,
           (  x1 = new XMLList('<a>b</a>'), (xml3 == x1)));

Assert.expectEq( "<>[list]</> == new XMLList([list])", true,
           (  x1 = new XMLList('<a b="1"><c>cat</c><d>walk</d></a><a b="2"><c>dog</c><d>run</d></a>'), (xml4 == x1)));
           
Assert.expectEq( "<></> == new XMLList()", true,
       (  x1 = new XMLList(), (xml5 == x1)));

Assert.expectEq( "<></> == new XMLList(\"\")", true,
       (  x1 = new XMLList(""), (xml5 == x1)));
           
Assert.expectEq( "<><{myVar}>hello</{myVar}></> == new XMLList('<value>hello</value>')", true,
       (  x1 = new XMLList("<kitty>hello</kitty>"), (xml6 == x1)));

Assert.expectEq( "<><c>&#x7B; \\\"\\\" &#x7B;</c></> == new XMLList(<c>{ \"\" }</c>)", true,
       (  x1 = new XMLList("<c q='1'>\"quotes\"</c><c q='2'>{curly brackets}</c>"), (xml7 == x1)));
       
Assert.expectEq( "<><a>5 &gt; 4</a></> == new XMLList('<a>5 > 4</a>')", true,
       (  x1 = new XMLList("<a>5 > 4</a>"), (xml8 == x1)));
       
Assert.expectEq( "Multiline XML", true,
       (  x1 = new XMLList(emplist), (empxml == x1)));
       
// Testing for extra directives. See bug 94230.
var xl = <><?xml version="1.0" encoding="UTF-8"?>
<?mso-infoPathSolution solutionVersion="1.0.0.26" productVersion="11.0.6250" PIVersion="1.0.0.0" href="file:///C:\Documents%20and%20Settings\Bob\BoB\Goodbye%20Doubt\Repository\CMS\Forms\PatternForm.xsn" name="urn:schemas-microsoft-com:office:infopath:PatternForm:urn-axiology-PatternDocument" language="en-us" ?>
<?mso-application progid="InfoPath.Document"?><root><blah>a</blah></root></>;
       
Assert.expectEq("Testing for extra directives", (<><root><blah>a</blah></root></>).toString(), xl.toString());

xl = new XMLList("<?xml version=\"1.0\" encoding=\"UTF-8\"?><?mso-infoPathSolution solutionVersion=\"1.0.0.26\" productVersion=\"11.0.6250\" PIVersion=\"1.0.0.0\" href=\"file:///C:\Documents%20and%20Settings\Bob\BoB\Goodbye%20Doubt\Repository\CMS\Forms\PatternForm.xsn\" name=\"urn:schemas-microsoft-com:office:infopath:PatternForm:urn-axiology-PatternDocument\" language=\"en-us\" ?><?mso-application progid=\"InfoPath.Document\"?><root><blah>a</blah></root>");

Assert.expectEq("Testing for extra directives", (new XMLList("<root><blah>a</blah></root>")).toString(), xl.toString());
           
END();
