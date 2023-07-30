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

START("13.4.4.17 - XML inScopeNamespaces()");

//TEST(1, true, XML.prototype.hasOwnProperty("inScopeNamespaces"));
 
x1 =
<alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
    <bravo>one</bravo>
</alpha>;

namespaces = x1.bravo.inScopeNamespaces();


TEST(2, "foo", namespaces[0].prefix);
TEST(3, "http://foo/", namespaces[0].uri);
TEST(4, "bar", namespaces[1].prefix);
TEST(5, "http://bar/", namespaces[1].uri);
//TEST(6, "", namespaces[2].prefix);
//TEST(7, "", namespaces[2].uri);
TEST(8, 2, namespaces.length);

var n1 = new Namespace('pfx', 'http://w3.org');
var n2 = new Namespace('http://us.gov');
var n3 = new Namespace('boo', 'http://us.gov');
var n4 = new Namespace('boo', 'http://hk.com');
var xml = "<a><b s='1'><c>55</c><d>bird</d></b><b s='2'><c>5</c><d>dinosaur</d></b></a>";


Assert.expectEq( "Two namespaces in toplevel scope:", ('http://hk.com'),
           (  x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n4), x1.inScopeNamespaces()[1].toString()));

Assert.expectEq( "Two namespaces in toplevel scope:", ('http://w3.org'),
           (  x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n4), x1.inScopeNamespaces()[0].toString()));

Assert.expectEq( "No namespace:", (''),
           (  x1 = new XML(xml), x1.inScopeNamespaces().toString()));
           
try {
    x1 = new XML(xml);
    x1.addNamespace();
    result = "no exception";
} catch (e1) {
    result = "exception";
}

Assert.expectEq( "Undefined namespace:", "exception", result);

Assert.expectEq( "Undefined namespace, length:", 1,
       (  x1 = new XML(xml), x1.addNamespace(null), x1.inScopeNamespaces().length));
       
Assert.expectEq( "One namespace w/o prefix, length:", 1,
       (  x1 = new XML(xml), x1.addNamespace(n2), x1.inScopeNamespaces().length));

Assert.expectEq( "One namespace w/ prefix, length:", 1,
       (  x1 = new XML(xml), x1.addNamespace(n1), x1.inScopeNamespaces().length));

Assert.expectEq( "One namespace at toplevel, one at child, length at toplevel:", 1,
       (  x1 = new XML(xml), x1.addNamespace(n3), x1.b[0].c.addNamespace(n4), x1.inScopeNamespaces().length));

Assert.expectEq( "One namespace at toplevel, two at child, length at child:", 2,
       (  x1 = new XML(xml), x1.addNamespace(n3), x1.b[1].c.addNamespace(n4), x1.b[1].c.addNamespace(n1), x1.b[1].c.inScopeNamespaces().length));

Assert.expectEq( "inScopeNamespaces[0].typeof:", "object",
           (  x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n4), typeof x1.inScopeNamespaces()[0]));

Assert.expectEq( "inScopeNamespaces[1].prefix:", "boo",
           (  x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n4), x1.inScopeNamespaces()[1].prefix));

   
var xmlDoc = "<?xml version=\"1.0\"?><xsl:stylesheet xmlns:xsl=\"http://www.w3.org/TR/xsl\"><b><c xmlns:foo=\"http://www.foo.org/\">hi</c></b></xsl:stylesheet>";

Assert.expectEq( "Reading one toplevel namespace:", (["http://www.w3.org/TR/xsl"]).toString(),
       (  x1 = new XML(xmlDoc), x1.inScopeNamespaces().toString()));

Assert.expectEq( "Reading two namespaces up parent chain:", (["http://www.foo.org/","http://www.w3.org/TR/xsl"]).toString(),
       (  x1 = new XML(xmlDoc), x1.b.c.inScopeNamespaces().toString()));

END();
