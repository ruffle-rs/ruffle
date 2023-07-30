/* -*- Mode: java; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 *
 * ***** BEGIN LICENSE BLOCK *****
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
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
 

START("13.2.5 - Properties of Namespace Instances");

n = new Namespace("ns", "http://someuri");
TEST(1, true, n.hasOwnProperty("prefix"));
TEST(2, true, n.hasOwnProperty("uri"));
TEST(3, true, n.propertyIsEnumerable("prefix"));
TEST(4, true, n.propertyIsEnumerable("uri"));

var prefixCount = 0;
var uriCount = 0;
var p;
for(p in n)
{
    if(p == "prefix") prefixCount++;
    if(p == "uri") uriCount++;
}

TEST(5, 1, prefixCount);
TEST(6, 1, uriCount);
TEST(7, "ns", n.prefix);
TEST(8, "http://someuri", n.uri);

var n = new Namespace();
Assert.expectEq( "Namespace.uri = blank", '', n.uri);

n = new Namespace('http://www.w3.org/TR/html4/');
Assert.expectEq( "Namespace.uri = http://www.w3.org/TR/html4/", 'http://www.w3.org/TR/html4/', n.uri);

n = new Namespace('pfx','http://www.w3.org/TR/html4/');
Assert.expectEq( "Namespace.uri = http://www.w3.org/TR/html4/", 'http://www.w3.org/TR/html4/', n.uri);

n = new Namespace('', '');
Assert.expectEq( "Namespace.uri = ''", '', n.uri);

// Make sure uri is read-only
var thisError = "no error";
try{
    n.uri = "hi";
}catch(e:ReferenceError){
    thisError = e.toString();
}finally{
    Assert.expectEq( "Trying to prove that Namespace.uri is read only", "ReferenceError: Error #1074", Utils.referenceError(thisError));
 }
Assert.expectEq( "Setting Namespace.uri", '', n.uri);

n = new Namespace('pfx','http://www.w3.org/TR/html4/');

    try{
    n.uri = "hi";
}catch(e:ReferenceError){
    thisError = e.toString();
}finally{
    Assert.expectEq( "Trying to prove that Namespace.uri is read only", "ReferenceError: Error #1074", Utils.referenceError(thisError));
 }

Assert.expectEq( "Setting Namespace.uri", 'http://www.w3.org/TR/html4/', n.uri);

n = new Namespace();
Assert.expectEq( "Namespace.prefix = blank", '', n.prefix);

n = new Namespace('http://www.w3.org/TR/html4/');
Assert.expectEq( "Namespace.prefix = blank", undefined, n.prefix);

n = new Namespace('pfx','http://www.w3.org/TR/html4/');
Assert.expectEq( "Namespace.prefix = pfx", 'pfx', n.prefix);

n = new Namespace('', '');
Assert.expectEq( "Namespace.prefix = ''", '', n.prefix);

// Make sure prefix is read-only
try{
    n.prefix = "hi";
}catch(e:ReferenceError){
    thisError = e.toString();
}finally{
    Assert.expectEq( "Trying to prove that Namespace.prefix is read only", "ReferenceError: Error #1074", Utils.referenceError(thisError));
 }
Assert.expectEq( "Setting Namespace.prefix", '', n.prefix);

n = new Namespace('pfx','http://www.w3.org/TR/html4/');
try{
    n.prefix = "hi";
}catch(e:ReferenceError){
    thisError = e.toString();
}finally{
    Assert.expectEq( "Trying to prove that Namespace.prefix is read only", "ReferenceError: Error #1074", Utils.referenceError(thisError));
 }
Assert.expectEq( "Setting Namespace.prefix", 'pfx', n.prefix);

n = new Namespace ("http://www.w3.org/XML/1998/namespace");

var myXML = <xliff version="1.0" xml:lang="de">
<file>
<header></header>
<body>
hello
</body>
</file>
</xliff>;

var foo = myXML.@n::lang;

Assert.expectEq("Using xml:lang namespace", "de", foo.toString());

Assert.expectEq("Getting name() of xml:lang namespace", "http://www.w3.org/XML/1998/namespace::lang", foo.name().toString());

Assert.expectEq("Getting uri of xml:lang namespace", "http://www.w3.org/XML/1998/namespace", foo.name().uri);


myXML = <xliff version="1.0" xml:space="false">
<file>
<header></header>
<body>
hello
</body>
</file>
</xliff>;

foo = myXML.@n::space;

Assert.expectEq("Using xml:space namespace", "false", foo.toString());

Assert.expectEq("Getting name() of xml:space namespace", "http://www.w3.org/XML/1998/namespace::space", foo.name().toString());

Assert.expectEq("Getting uri of xml:space namespace", "http://www.w3.org/XML/1998/namespace", foo.name().uri);


namespace foot = "bar";
xn = new Namespace ("p", "y");

var for_in_values = [];
for (var nn in foot)
{
    for_in_values.push(nn);
}

Assert.expectEq("Prefix in for-in loop", "prefix", for_in_values[1]);
//Assert.expectEq("Prefix in for-in loop BUG 125319", false, (for_in_values[1] == "prefix"));
Assert.expectEq("URI in for-in loop", "uri", for_in_values[0]);
//Assert.expectEq("URI in for-in loop BUG 125316", false, (for_in_values[0] == "uri"));

var for_each_values = [];
for each (nn in foot)
{
    for_each_values.push(nn);
}

Assert.expectEq("Prefix in for-each loop", "bar", for_each_values[0]);
//Assert.expectEq("Prefix in for-each loop BUG 125316", false, (for_each_values[0] == "bar"));
Assert.expectEq("URI in for-each loop", undefined, for_each_values[1]);

for_each_values = [];
for each (nn in xn)
{
    for_each_values.push(nn);
}

Assert.expectEq("Prefix in for-each loop", "y", for_each_values[0]);
Assert.expectEq("URI in for-each loop", "p", for_each_values[1]);

END();
