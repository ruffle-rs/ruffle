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
 

START("13.4.4.29 - XML prependChild()");

//TEST(1, true, XML.prototype.hasOwnProperty("prependChild"));

x1 =
<alpha>
    <bravo>
        <charlie>one</charlie>
    </bravo>
</alpha>;

correct =
<alpha>
    <bravo>
        <foo>bar</foo>
        <charlie>one</charlie>
    </bravo>
</alpha>;

x1.bravo.prependChild(<foo>bar</foo>);

TEST(2, correct, x1);

emps =
<employees>
    <employee>
        <name>John</name>
    </employee>
    <employee>
        <name>Sue</name>
    </employee>
</employees>

correct =
<employees>
    <employee>
        <prefix>Mr.</prefix>
        <name>John</name>
    </employee>
    <employee>
        <name>Sue</name>
    </employee>
</employees>

emps.employee.(name == "John").prependChild(<prefix>Mr.</prefix>);

TEST(3, correct, emps);

XML.prettyPrinting = false;
var xmlDoc = "<company></company>";
var child1 = new XML("<employee id='1'><name>John</name></employee>");
var child2 = new XML("<employee id='2'><name>Sue</name></employee>");
var child3 = new XML("<employee id='3'><name>Bob</name></employee>");

Assert.expectEq( "MYXML = new XML(xmlDoc), MYXML.prependChild(child1), MYXML.toString()",
    "<company><employee id=\"1\"><name>John</name></employee></company>",
    (MYXML = new XML(xmlDoc), MYXML.prependChild(child1), MYXML.toString()));

var MYXML = new XML(xmlDoc);
MYXML.prependChild(child2);

Assert.expectEq( "MYXML.prependChild(child1), MYXML.toString()",
    "<company><employee id=\"1\"><name>John</name></employee><employee id=\"2\"><name>Sue</name></employee></company>",
    (MYXML.prependChild(child1), MYXML.toString()));

MYXML = new XML(xmlDoc);
MYXML.prependChild(child3);
MYXML.prependChild(child2);

Assert.expectEq ("Making sure child added is a duplicate", true, (child2 === MYXML.children()[0]));
Assert.expectEq ("Making sure child added is a true copy", true, (child2 == MYXML.children()[0]));

Assert.expectEq( "MYXML.prependChild(child1), MYXML.toString()",
    "<company><employee id=\"1\"><name>John</name></employee><employee id=\"2\"><name>Sue</name></employee><employee id=\"3\"><name>Bob</name></employee></company>",
    (MYXML.prependChild(child1), MYXML.toString()));

Assert.expectEq( "MYXML.prependChild('simple text string'), MYXML.toString()",
    "<company>simple text string<employee id=\"1\"><name>John</name></employee><employee id=\"2\"><name>Sue</name></employee><employee id=\"3\"><name>Bob</name></employee></company>",
    (MYXML.prependChild("simple text string"), MYXML.toString()));

// !!@ test cases for adding comment nodes, PI nodes

MYXML = new XML(xmlDoc);
MYXML.prependChild(child3);
MYXML.prependChild(child2);
MYXML.prependChild(child1);

var expectedResult;
expectedResult = '<company><employee id="1"><name>John</name></employee><employee id="2"><name>Sue</name></employee><employee id="3"><name>Bob</name></employee></company>';

Assert.expectEq( "MYXML.prependChild('<!-- comment -->'), MYXML.toString()",
    expectedResult,
    (MYXML.prependChild("<!-- comment -->"), MYXML.toString()));

XML.ignoreComments = false;
MYXML = new XML(xmlDoc);
MYXML.prependChild(child3);
MYXML.prependChild(child2);
MYXML.prependChild(child1);

expectedResult = '<company><!-- comment --><employee id="1"><name>John</name></employee><employee id="2"><name>Sue</name></employee><employee id="3"><name>Bob</name></employee></company>';

Assert.expectEq( "MYXML.prependChild('<!-- comment -->'), MYXML.toString()",
    expectedResult,
    (MYXML.prependChild("<!-- comment -->"), MYXML.toString()));

MYXML = new XML(xmlDoc);
MYXML.prependChild(child3);
MYXML.prependChild(child2);
MYXML.prependChild(child1);

expectedResult = '<company><employee id="1"><name>John</name></employee><employee id="2"><name>Sue</name></employee><employee id="3"><name>Bob</name></employee></company>';

Assert.expectEq( "MYXML.prependChild('<?xml-stylesheet href=\"classic.xsl\" type=\"text/xml\"?>'), MYXML.toString()",
    expectedResult,
    (MYXML.prependChild("<?xml-stylesheet href=\"classic.xsl\" type=\"text/xml\"?>"), MYXML.toString()));

XML.ignoreProcessingInstructions = false;
MYXML = new XML(xmlDoc);
MYXML.prependChild(child3);
MYXML.prependChild(child2);
MYXML.prependChild(child1);

var expected = "TypeError: error: XML declaration may only begin entities.";
var result = "error, exception not thrown";

expectedResult = '<company><?xml-stylesheet href="classic.xsl" type="text/xml"?><employee id="1"><name>John</name></employee><employee id="2"><name>Sue</name></employee><employee id="3"><name>Bob</name></employee></company>';

Assert.expectEq( "MYXML.prependChild('<?xml-stylesheet href=\"classic.xsl\" type=\"text/xml\"?>'), MYXML.toString()",
    expectedResult,
    (MYXML.prependChild("<?xml-stylesheet href=\"classic.xsl\" type=\"text/xml\"?>"), MYXML.toString()));

try{

MYXML.prependChild("<?xml version=\"1.0\"?>");

} catch( e1 ){

result = e1.toString();

}
//Taking test out because bug 108406 has been deferred.
//Assert.expectEq("OPEN BUG: 108406 - MYXML.prependChild(\"<?xml version=\"1.0\"?>\")", expected, result);

// !!@ setting a property that starts with an "@" that implies an attribute name??
// Rhino throws an error

MYXML = new XML(xmlDoc);
MYXML.prependChild(child3);
MYXML.prependChild(child2);
MYXML.prependChild(child1);

expectedResult = '<company><@notanattribute>hi</@notanattribute><employee id="1"><name>John</name></employee><employee id="2"><name>Sue</name></employee><employee id="3"><name>Bob</name></employee></company>';

Assert.expectEq( "MYXML.prependChild(\"<@notanattribute>hi</@notanattribute>\"), MYXML.toString()",
    expectedResult,
    (MYXML.prependChild("<@notanattribute>hi</@notanattribute>"), MYXML.toString()));

MYXML = new XML('<LEAGUE></LEAGUE>');
x1 = new XMLList('<TEAM t="a">Giants</TEAM><TEAM t="b">Robots</TEAM>');
MYXML.prependChild(x1);
            
Assert.expectEq( "Prepend XMLList",
            '<LEAGUE><TEAM t="a">Giants</TEAM><TEAM t="b">Robots</TEAM></LEAGUE>',
            (MYXML.toString()) );
            
MYXML = new XML('<SCARY><MOVIE></MOVIE></SCARY>');
x1 = "poltergeist";
MYXML.MOVIE.prependChild(x1);
            
Assert.expectEq( "Prepend a string to child node",
            '<SCARY><MOVIE>poltergeist</MOVIE></SCARY>',
            (MYXML.toString()) );

// I believe the following two test cases are wrong. See bug 145184.
            
MYXML = new XML('<SCARY><MOVIE></MOVIE></SCARY>');
x1 = "poltergeist";
MYXML.prependChild(x1);
            
Assert.expectEq( "Prepend a string to top node",
            '<SCARY>poltergeist<MOVIE/></SCARY>',
            (MYXML.toString()) );
            
MYXML = new XML('<SCARY><MOVIE></MOVIE></SCARY>');
x1 = new XML("<the>poltergeist</the>");
MYXML.prependChild(x1);
            
Assert.expectEq( "Prepend a node to child node",
            '<SCARY><the>poltergeist</the><MOVIE/></SCARY>',
            (MYXML.toString()) );

var a = <a><b><c/></b></a>;

try {
    a.b.prependChild (a);
    result = a;
} catch (e1) {
    result = Utils.typeError(e1.toString());
}

Assert.expectEq("a = <a><b><c/></b></a>, a.b.prependChild(a)", "TypeError: Error #1118", result);



END();
