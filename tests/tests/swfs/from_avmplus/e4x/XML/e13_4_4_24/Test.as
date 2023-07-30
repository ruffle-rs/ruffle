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

START("13.4.4.24 - XML namespaceDeclarations()");

//TEST(1, true, XML.prototype.hasOwnProperty("namespaceDeclarations"));
    
x1 =
<foo:alpha xmlns:foo="http://foo/" xmlns:bar="http://bar/">
    <foo:bravo>one</foo:bravo>
</foo:alpha>;

y1 = x1.namespaceDeclarations();

TEST(2, 2, y1.length);
TEST(3, "object", typeof(y1[0]));
TEST(4, "object", typeof(y1[1]));
TEST(5, "foo", y1[0].prefix);
TEST(6, Namespace("http://foo/"), y1[0]);
TEST(7, "bar", y1[1].prefix);
TEST(8, Namespace("http://bar/"), y1[1]);

var n1 = new Namespace('pfx', 'http://w3.org');
var n2 = new Namespace('http://us.gov');
var n3 = new Namespace('boo', 'http://us.gov');
var n4 = new Namespace('boo', 'http://hk.com');
var xml1 = "<a><b s='1'><c>55</c><d>bird</d></b><b s='2'><c>5</c><d>dinosaur</d></b></a>";

//Assert.expectEq( "Undefined declaration:", (''),
//     (  x1 = new XML(xml1), x1.addNamespace(), x1.namespaceDeclarations().toString()));

Assert.expectEq( "Undefined declaration, length:", 0,
       (  x1 = new XML(xml1), x1.addNamespace(null), x1.namespaceDeclarations().length));

Assert.expectEq( "Two declarations in toplevel scope:", ('http://hk.com'),
           (  x1 = new XML(xml1), x1.addNamespace(n1), x1.addNamespace(n4), x1.namespaceDeclarations()[1].toString()));

Assert.expectEq( "Two declarations in toplevel scope:", ('http://w3.org'),
           (  x1 = new XML(xml1), x1.addNamespace(n1), x1.addNamespace(n4), x1.namespaceDeclarations()[0].toString()));

Assert.expectEq( "No declaration:", (''),
           (  x1 = new XML(xml1), x1.namespaceDeclarations().toString()));
           
Assert.expectEq( "One declaration w/o prefix, length:", 0,
       (  x1 = new XML(xml1), x1.addNamespace(n2), x1.namespaceDeclarations().length));

Assert.expectEq( "One declaration w/ prefix, length:", 1,
       (  x1 = new XML(xml1), x1.addNamespace(n1), x1.namespaceDeclarations().length));

Assert.expectEq( "One declaration at toplevel, one at child, length at toplevel:", 1,
       (  x1 = new XML(xml1), x1.addNamespace(n3), x1.b[0].c.addNamespace(n4), x1.namespaceDeclarations().length));

Assert.expectEq( "One declaration at toplevel, two at child, length at child:", 2,
       (  x1 = new XML(xml1), x1.addNamespace(n3), x1.b[1].c.addNamespace(n4), x1.b[1].c.addNamespace(n1), x1.b[1].c.namespaceDeclarations().length));

Assert.expectEq( "namespaceDeclarations[1].typeof:", "object",
           (  x1 = new XML(xml1), x1.addNamespace(n1), x1.addNamespace(n4), typeof x1.namespaceDeclarations()[1]));

Assert.expectEq( "namespaceDeclarations[1].prefix:", "boo",
           (  x1 = new XML(xml1), x1.addNamespace(n1), x1.addNamespace(n4), x1.namespaceDeclarations()[1].prefix));

           
var xml1Doc = "<?xml1 version=\"1.0\"?><xsl:stylesheet xmlns:xsl=\"http://www.w3.org/TR/xsl\"><b><c xmlns:foo=\"http://www.foo.org/\">hi</c></b></xsl:stylesheet>";

Assert.expectEq( "Reading one toplevel declaration:", "http://www.w3.org/TR/xsl",
       (  x1 = new XML(xml1Doc), x1.namespaceDeclarations().toString()));

Assert.expectEq( "Another declaration up parent chain:", "http://www.foo.org/",
       (  x1 = new XML(xml1Doc), x1.b.c.namespaceDeclarations().toString()));
       
// Adding because of Mozilla bug https://bugzilla.mozilla.org/show_bug.cgi?id=278112

var xhtml1NS = new Namespace('http://www.w3.org/1999/xhtml');
var xhtml = <html />;

Assert.expectEq("namespaceDeclarations before setNamespace()", 0, xhtml.namespaceDeclarations().length);

xhtml.setNamespace(xhtml1NS);

Assert.expectEq("namespaceDeclarations after setNamespace()", 0, xhtml.namespaceDeclarations().length);


END();
