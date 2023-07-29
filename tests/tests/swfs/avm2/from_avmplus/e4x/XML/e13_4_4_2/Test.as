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
 

START("13.4.4.2 - XML addNamespace()");

//TEST(1, true, XML.prototype.hasOwnProperty("addNamespace"));
//TEST(2, true, XML.prototype.hasOwnProperty("children"));

e =
<employees>
    <employee id="0"><name>Jim</name><age>25</age></employee>
    <employee id="1"><name>Joe</name><age>20</age></employee>
</employees>;

n = "http://foobar/";
e.addNamespace(n);

n = new Namespace();
e.addNamespace(n);

n = new Namespace("http://foobar/");
e.addNamespace(n);

x1 = <a/>;
n = new Namespace("ns", "uri");
x1.addNamespace(n);
TEST(2, "<a xmlns:ns=\"uri\"/>", x1.toXMLString());

var n1 = new Namespace('pfx', 'http://w3.org');
var n2 = new Namespace('http://us.gov');
var n3 = new Namespace('boo', 'http://us.gov');
var n4 = new Namespace('boo', 'http://hk.com');
var xml = "<a><b s='1'><c>55</c><d>bird</d></b><b s='2'><c>5</c><d>dinosaur</d></b></a>";
var xmlwithns = "<a xmlns:pfx=\"http://w3.org\"><a><b s='1'><c>55</c><d>bird</d></b><b s='2'><c>5</c><d>dinosaur</d></b></a>";

Assert.expectEq( "addNamespace with prefix:", "http://w3.org",
           (  x1 = new XML(xml), x1.addNamespace(n1), myGetNamespace(x1,'pfx').toString()));

Assert.expectEq( "addNamespace without prefix:", undefined,
           (  x1 = new XML(xml), x1.addNamespace(n2), myGetNamespace(x1, 'blah')));

expectedStr = "ArgumentError: Error #1063: Argument count mismatch on XML/addNamespace(). Expected 1, got 0";
expected = "Error #1063";
actual = "No error";

try {
    x1.addNamespace();
} catch(e1) {
    actual = Utils.grabError(e1, e1.toString());
}

Assert.expectEq( "addNamespace(): Error. Needs 1 argument", expected, actual);

Assert.expectEq( "Does namespace w/o prefix change XML object:", true,
           (  x1 = new XML(xml), y1 = new XML(xml), x1.addNamespace(n1), (x1 == y1)));

Assert.expectEq( "Does namespace w/ prefix change XML object:", true,
           (  x1 = new XML(xml), y1 = new XML(xml), x1.addNamespace(n2), (x1 == y1)));

Assert.expectEq( "Adding two different namespaces:", 'http://w3.org',
       (  x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n3), myGetNamespace(x1, 'pfx').toString()));

Assert.expectEq( "Adding two different namespaces:", 'http://us.gov',
           (  x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n3), myGetNamespace(x1, 'boo').toString()));

Assert.expectEq( "Adding namespace with pre-existing prefix:", 'http://hk.com',
           (  x1 = new XML(xml), x1.addNamespace(n3), x1.addNamespace(n4), myGetNamespace(x1, 'boo').toString()));


Assert.expectEq( "Adding namespace to something other than top node:", 'http://hk.com',
           (  x1 = new XML(xml), x1.b[0].d.addNamespace(n4), myGetNamespace(x1.b[0].d, 'boo').toString()));


Assert.expectEq( "Adding namespace to XMLList element:", 'http://hk.com',
           (  x1 = new XMLList(xml), x1.b[1].addNamespace(n4), myGetNamespace(x1.b[1], 'boo').toString()));
           


// namespaces with prefixes are preserved

x2 = <ns2:x xmlns:ns2="foo"><b>text</b></ns2:x>;
x2s = x2.toString();
correct = '<ns2:x xmlns:ns2="foo">\n  <b>text</b>\n</ns2:x>';
Assert.expectEq("Original XML", x2s, correct);

// Adding a namespace to a node will clear a conflicting prefix
var ns = new Namespace ("ns2", "newuri");
x2.addNamespace (ns);
x2s = x2.toString();
 
correct = '<x xmlns:ns2="newuri" xmlns="foo">\n  <b>text</b>\n</x>';

Assert.expectEq("Adding namespace that previously existed with a different prefix", correct,
       x2s);



END();
