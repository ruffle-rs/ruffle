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

START("13.4.4.31 - XML removeNamespace()");

//TEST(1, true, XML.prototype.hasOwnProperty("removeNamespace"));

x1 =
<alpha xmlns:foo="http://foo/">
    <bravo>one</bravo>
</alpha>;

correct =
<alpha>
    <bravo>one</bravo>
</alpha>;

// TEST(1.5, correct, x1);
TEST(1.6, false, (correct.toString() == x1.toString()));

x1.removeNamespace("http://foo/");

TEST(2, correct.toString(), x1.toString());

TEST(2.5, correct, x1);

// Shouldn't remove namespace if referenced
x1 =
<foo:alpha xmlns:foo="http://foo/">
    <bravo>one</bravo>
</foo:alpha>;

correct =
<foo:alpha xmlns:foo="http://foo/">
    <bravo>one</bravo>
</foo:alpha>;

x1.removeNamespace("http://foo/");

// TEST(3, correct, x1);


var xmlDoc = "<?xml version=\"1.0\"?><xsl:stylesheet xmlns:xsl=\"http://www.w3.org/TR/xsl\"><b><c xmlns:foo=\"http://www.foo.org/\">hi</c></b></xsl:stylesheet>";
var ns1 = Namespace ("xsl", "http://www.w3.org/TR/xsl");
var ns2 = Namespace ("foo", "http://www.foo.org");


// Namespace that is referenced by QName should not be removed
Assert.expectEq( "MYXML.removeNamespace(QName reference)", "http://www.w3.org/TR/xsl",
       (  MYXML = new XML(xmlDoc), MYXML.removeNamespace('xsl'), myGetNamespace(MYXML, 'xsl').toString()));


// Other namespaces should be removed
Assert.expectEq( "MYXML.removeNamespace(no Qname reference)", undefined,
       (  MYXML = new XML(xmlDoc), MYXML.b.c.removeNamespace('foo'), myGetNamespace(MYXML, 'foo')) );

var n1 = new Namespace('pfx', 'http://w3.org');
var n2 = new Namespace('http://us.gov');
var n3 = new Namespace('boo', 'http://us.gov');
var n4 = new Namespace('boo', 'http://hk.com');
var xml = "<a><b s='1'><c>55</c><d>bird</d></b><b s='2'><c>5</c><d>dinosaur</d></b></a>";

Assert.expectEq("Two namespaces in one node", true,
           ( x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n3), x1.removeNamespace(n3),
             x1.removeNamespace(n1), (myGetNamespace(x1, 'pfx') == myGetNamespace(x1, 'boo'))) );

Assert.expectEq("Two namespaces in one node", 1,
           ( x1 = new XML(xml), x1.addNamespace(n1), x1.addNamespace(n3), x1.removeNamespace(n3),
             x1.removeNamespace(n1), x1.inScopeNamespaces().length) );
             
Assert.expectEq("Two namespace in two different nodes", undefined,
           ( x1 = new XML(xml), x1.addNamespace(n3), x1.b[0].c.addNamespace(n1),
             x1.removeNamespace(n1), myGetNamespace(x1.b[0].c, 'pfx')));
             
var xml1 = <a xmlns:n2="http://us.gov"><b s='1'><c>55</c><d>bird</d></b><b s='2'><c>5</c><d>dinosaur</d></b></a>;
var xml2 = <a><b s='1'><c>55</c><d>bird</d></b><b s='2'><c>5</c><d>dinosaur</d></b></a>;
             
Assert.expectEq("Remove namespace without prefix: " + xml2,
             xml1.removeNamespace(n2), xml1);


END();
