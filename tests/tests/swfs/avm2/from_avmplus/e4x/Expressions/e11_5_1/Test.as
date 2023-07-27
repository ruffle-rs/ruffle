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

START("11.5.1 - Equality Operators");

x1 = <alpha>one</alpha>;
y1 = <alpha>one</alpha>;
TEST(1, true, (x1 == y1) && (y1 == x1));

var myxml:XML = <a>foo</a>;
var str1:String = "foo";
TEST(17, true, myxml.hasSimpleContent());
TEST(18, true, myxml==str1);


// Should return false if comparison is not XML
y1 = "<alpha>one</alpha>";
TEST(2, false, (x1 == y1) || (y1 == x1));

y1 = undefined
TEST(3, false, (x1 == y1) || (y1 == x1));

y1 = null
TEST(4, false, (x1 == y1) || (y1 == x1));

// Should check logical equiv.
x1 = <alpha attr1="value1">one<bravo attr2="value2">two</bravo></alpha>;
y1 = <alpha attr1="value1">one<bravo attr2="value2">two</bravo></alpha>;
TEST(5, true, (x1 == y1) && (y1 == x1));

y1 = <alpha attr1="new value">one<bravo attr2="value2">two</bravo></alpha>;
TEST(6, false, (x1 == y1) || (y1 == x1));

m = new Namespace();
n = new Namespace();
TEST(7, true, m == n);

m = new Namespace("uri");
TEST(8, false, m == n);

n = new Namespace("ns", "uri");
TEST(9, true, m == n);

m = new Namespace(n);
TEST(10, true, m == n);

TEST(11, false, m == null);
TEST(12, false, null == m);

m = new Namespace("ns", "http://anotheruri");
TEST(13, false, m == n);

p = new QName("a");
q = new QName("b");
TEST(14, false, p == q);

q = new QName("a");
TEST(15, true, p == q);

q = new QName("http://someuri", "a");
TEST(16, false, p == q);

q = new QName(null, "a");
TEST(16, false, p == q);

var x1  = new XML("<aa><a>A</a><a>B</a><a>C</a></aa>");
var y0 = new XML("<a><b>A</b><c>B</c></a>");
var y1 = new XML("<aa><a>A</a><a>B</a><a>C</a></aa>");
var y2 = new XML("<bb><b>A</b><b>B</b><b>C</b></bb>");
var y3 = new XML("<aa><a>Dee</a><a>Eee</a><a>Fee</a></aa>");

Assert.expectEq( "x=XMLList,                y=XML                   :", false, (x1==y0) );
Assert.expectEq( "x=XMLList,                y=XMLList               :", true,  (x1==y1) );
Assert.expectEq( "x=XMLList,                y=XMLList               :", false, (x1==y2) );
Assert.expectEq( "x=XMLList,                y=XMLList               :", false, (x1==y3) );


var xt = new XML("<a><b>text</b></a>");
var xa = new XML("<a attr='ATTR'><b>attribute</b></a>");
var xh = new XML("<a>hasSimpleContent</a>");
var yt = new XML("<a><b>text</b></a>");
var ya = new XML("<a attr='ATTR'><b>attribute</b></a>");
var yh = new XML("<a>hasSimpleContent</a>");

Assert.expectEq( "x.[[Class]]='text,        y.[[Class]]='text'      :", true,  (xt==yt) );
Assert.expectEq( "x.[[Class]]='text,        y.[[Class]]='attribute' :", false, (xt==ya.@attr) );
Assert.expectEq( "x.[[Class]]='text,        y.hasSimpleContent()    :", false, (xt==yh) );

Assert.expectEq( "x.[[Class]]='attribute,   y.[[Class]]='text'      :", false, (xa.@attr==yt) );
Assert.expectEq( "x.[[Class]]='attribute,   y.[[Class]]='attribute' :", true,  (xa.@attr==ya.@attr) );
Assert.expectEq( "x.[[Class]]='attribute,   y.hasSimpleContent()    :", false, (xa.@attr==yh) );

Assert.expectEq( "x.hasSimpleContent(),     y.[[Class]]='text'      :", false, (xh==yt) );
Assert.expectEq( "x.hasSimpleContent(),     y.[[Class]]='attribute' :", false, (xh==ya.@attr) );
Assert.expectEq( "x.hasSimpleContent(),     y.hasSimpleContent()    :", true,  (xh==yh) );


var xqn0  = new QName("n0");
var xqn1  = new QName("ns1","n1");

var yqn00 = new QName("n0");
var yqn01 = new QName("nA");
var yqn10 = new QName("ns1", "n1" );
var yqn11 = new QName("ns1", "nB");
var yqn12 = new QName("nsB","n1" );
var yqn13 = new QName("nsB","nB");

Assert.expectEq( "QName('n0'),              QName('n0')              :", true,  (xqn0==yqn00) );
Assert.expectEq( "QName('n0'),              QName('nA')              :", false, (xqn0==yqn01) );
Assert.expectEq( "QName('n0'),              QName('ns1','n1')        :", false, (xqn0==yqn10) );
Assert.expectEq( "QName('n0'),              QName('ns1','nB')        :", false, (xqn0==yqn11) );
Assert.expectEq( "QName('n0'),              QName('nsB','n1')        :", false, (xqn0==yqn12) );
Assert.expectEq( "QName('n0'),              QName('naB','nB')        :", false, (xqn0==yqn13) );

Assert.expectEq( "QName('ns1','n1'),        QName('n0')              :", false, (xqn1==yqn00) );
Assert.expectEq( "QName('ns1','n1'),        QName('nA')              :", false, (xqn1==yqn01) );
Assert.expectEq( "QName('ns1','n1'),        QName('ns1','n1')        :", true,  (xqn1==yqn10) );
Assert.expectEq( "QName('ns1','n1'),        QName('ns1','nB')        :", false, (xqn1==yqn11) );
Assert.expectEq( "QName('ns1','n1'),        QName('nsB','n1')        :", false, (xqn1==yqn12) );
Assert.expectEq( "QName('ns1','n1'),        QName('nsB','nB')        :", false, (xqn1==yqn13) );


var xns0  = new Namespace();
var xns1  = new Namespace("uri1");
var xns2  = new Namespace("pre2","uri2");

var yns00 = new Namespace();
var yns10 = new Namespace("uri1");
var yns11 = new Namespace("uriB");
var yns20 = new Namespace("pre2","uri2");
var yns21 = new Namespace("pre2","uriC");
var yns22 = new Namespace("preC","uri2");
var yns23 = new Namespace("preC","uriC");


Assert.expectEq( "Namespace(),              Namespace()              :", true,  (xns0==yns00) );
Assert.expectEq( "Namespace(),              Namespace('uri1')        :", false, (xns0==yns10) );
Assert.expectEq( "Namespace(),              Namespace('uriB')        :", false, (xns0==yns11) );
Assert.expectEq( "Namespace(),              Namespace('pre2','uri2') :", false, (xns0==yns20) );
Assert.expectEq( "Namespace(),              Namespace('pre2','uriC') :", false, (xns0==yns21) );
Assert.expectEq( "Namespace(),              Namespace('preC','uri2') :", false, (xns0==yns22) );
Assert.expectEq( "Namespace(),              Namespace('preC','uriC') :", false, (xns0==yns23) );

Assert.expectEq( "Namespace('uri1'),        Namespace()              :", false, (xns1==yns00) );
Assert.expectEq( "Namespace('uri1'),        Namespace('uri1')        :", true,  (xns1==yns10) );
Assert.expectEq( "Namespace('uri1'),        Namespace('uriB')        :", false, (xns1==yns11) );
Assert.expectEq( "Namespace('uri1'),        Namespace('pre2','uri2') :", false, (xns1==yns20) );
Assert.expectEq( "Namespace('uri1'),        Namespace('pre2','uriC') :", false, (xns1==yns21) );
Assert.expectEq( "Namespace('uri1'),        Namespace('preC','uri2') :", false, (xns1==yns22) );
Assert.expectEq( "Namespace('uri1'),        Namespace('preC','uriC') :", false, (xns1==yns23) );

Assert.expectEq( "Namespace('pre2','uri2'), Namespace()              :", false, (xns2==yns00) );
Assert.expectEq( "Namespace('pre2','uri2'), Namespace('uri1')        :", false, (xns2==yns10) );
Assert.expectEq( "Namespace('pre2','uri2'), Namespace('uriB')        :", false, (xns2==yns11) );
Assert.expectEq( "Namespace('pre2','uri2'), Namespace('pre2','uri2') :", true,  (xns2==yns20) );
Assert.expectEq( "Namespace('pre2','uri2'), Namespace('pre2','uriC') :", false, (xns2==yns21) );
Assert.expectEq( "Namespace('pre2','uri2'), Namespace('preC','uri2') :", true,  (xns2==yns22) );
Assert.expectEq( "Namespace('pre2','uri2'), Namespace('preC','uriC') :", false, (xns2==yns23) );


END();
