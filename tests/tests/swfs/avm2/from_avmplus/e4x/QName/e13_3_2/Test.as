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

START("13.3.2 - QName Constructor");

q = new QName("*");
TEST(1, "object", typeof(q));
TEST(2, "*", q.localName);
TEST(3, null, q.uri);
TEST(4, "*::*", q.toString());

// Default namespace
q = new QName("foobar");
TEST(5, "object", typeof(q));
TEST(6, "foobar", q.localName);
TEST(7, "", q.uri);
TEST(8, "foobar", q.toString());

q = new QName("http://foobar/", "foobar");
TEST(9, "object", typeof(q));
TEST(10, "foobar", q.localName);
TEST(11, "http://foobar/", q.uri);
TEST(12, "http://foobar/::foobar", q.toString());

p = new QName(q);
TEST(13, typeof(p), typeof(q));
TEST(14, q.localName, p.localName);
TEST(15, q.uri, p.uri);

n = new Namespace("http://foobar/");
q = new QName(n, "foobar");
TEST(16, "object", typeof(q));

q = new QName(null);
TEST(17, "object", typeof(q));
TEST(18, "null", q.localName);
TEST(19, "", q.uri);
TEST(20, "null", q.toString());

q = new QName(null, null);
TEST(21, "object", typeof(q));
TEST(22, "null", q.localName);
TEST(23, null, q.uri);
TEST(24, "*::null", q.toString());

q = new QName("attr1");
q2 = new QName(q, "attr1");
q3 = QName(q);

TEST(25, "attr1", q.toString());
TEST(26, "attr1", q2.toString());
TEST(27, "attr1", q3.toString());

q = new QName(n, "attr1");
q2 = new QName(q, "attr1");

TEST(28, "http://foobar/::attr1", q.toString());
TEST(29, "http://foobar/::attr1", q2.toString());

// no value is supplied
Assert.expectEq( "ns = new QName()", "", (ns = new QName(), ns.localName) );

// one value is supplied
Assert.expectEq( "typeof new QName('name')", 'object', typeof new QName('name') );
Assert.expectEq( "new QName('name') instanceof QName", true, new QName('name') instanceof QName);
Assert.expectEq( "new QName('name') == 'name'", true, new QName('name') == 'name');
Assert.expectEq( "ns = new QName('name'), ns.uri == ''", true,
    (ns = new QName('name'), ns.uri == '') );
Assert.expectEq( "ns = new QName('name'), ns.uri == null", false,
    (ns = new QName('name'), ns.uri == null) );
Assert.expectEq( "ns = new QName('name'), ns.uri == undefined", false,
    (ns = new QName('name'), ns.uri == undefined) );
Assert.expectEq( "ns = new QName('name'), typeof ns.uri", 'string',
    (ns = new QName('name'), typeof ns.uri) );
Assert.expectEq( "ns = new QName('name'), ns.localName == 'name'", true,
    (ns = new QName('name'), ns.localName == 'name') );
Assert.expectEq( "ns = new QName(undefined)", "", (ns = new QName(undefined), ns.localName) );
Assert.expectEq( "ns = new QName('')", "", (ns = new QName(""), ns.localName) );
Assert.expectEq( "MYOB = new QName('nameofobj'),typeof new QName(MYOB)",
    'object',
    (MYOB = new QName('nameofobj'), typeof new QName(MYOB)) );


//two values are supplied
Assert.expectEq( "MYOB = new QName(null, 'nameofobj'); MYOB.toString()",
            "*::nameofobj",
             (MYOB = new QName(null, 'nameofobj'), MYOB.toString() ));

Assert.expectEq( "MYOB = new QName(null, 'nameofobj'); MYOB.uri", null,
             (MYOB = new QName(null, 'nameofobj'), MYOB.uri) );

Assert.expectEq( "MYOB = new QName(null, 'nameofobj'); MYOB.localName", 'nameofobj',
             (MYOB = new QName(null, 'nameofobj'), MYOB.localName) );
Assert.expectEq( "MYOB = new QName('namespace', undefined); MYOB.localName", "",
             (MYOB = new QName('namespace', undefined), MYOB.localName) );

Assert.expectEq( "MYOB = new QName('namespace', ''); MYOB.localName", "",
             (MYOB = new QName('namespace', ""), MYOB.localName) );
             
x1 =
<alpha>
    <bravo attr1="value1" ns:attr1="value3" xmlns:ns="http://someuri">
        <charlie attr1="value2" ns:attr1="value4"/>
    </bravo>
</alpha>
             
y = <ns:attr1 xmlns:ns="http://someuri"/>
q3 = y.name();

Assert.expectEq("q3 = y.name()", "http://someuri::attr1", q3.toString());
Assert.expectEq("x1.bravo.@[q3]", (new XML("value3")).toString(), x1.bravo.@[q3].toString());


END();
