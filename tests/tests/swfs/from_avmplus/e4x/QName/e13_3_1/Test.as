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

START("13.3.1 - QName Constructor as a Function");

q = QName("foobar");
p = new QName("foobar");
TEST(1, typeof(p), typeof(q));
TEST(2, p.localName, q.localName);
TEST(3, p.uri, q.uri);

q = QName("http://foobar/", "foobar");
p = new QName("http://foobar/", "foobar");
TEST(4, typeof(p), typeof(q));
TEST(5, p.localName, q.localName);
TEST(6, p.uri, q.uri);

p1 = QName(q);
p2 = new QName(q);
TEST(7, typeof(p2), typeof(p1));
TEST(8, p2.localName, p1.localName);
TEST(9, p2.uri, p1.uri);

n = new Namespace("http://foobar/");
q = QName(n, "foobar");
p = QName(n, "foobar");
TEST(10, typeof(p), typeof(q));
TEST(11, p.localName, q.localName);
TEST(12, p.uri, q.uri);

p = QName(q);
TEST(13, p, q);

// One value is supplied
Assert.expectEq( "QName('name').valueOf().toString()", 'name', QName('name').valueOf().toString() );
Assert.expectEq( "QName('name').valueOf() == 'name'", true, QName('name').valueOf() == 'name' );
Assert.expectEq( "typeof QName('name')", "object", typeof QName('name') );
//Assert.expectEq( "QName('name').__proto__", Namespace.prototype, QName('name').__proto__ );

// If one parameter is QName, same value is returned
Assert.expectEq ("foo = QName('foo'), bar = Qname(foo), bar === foo", true,
    (foo = QName("foo"), bar = QName(foo), bar === foo));

// If one parameter is QName but there is a Namespace param, different object is returned
Assert.expectEq ("foo = QName('foo'), bar = Qname(\"\", foo), bar === foo", false,
    (foo = QName("foo"), bar = QName("", foo), bar === foo));


//Two values are supplied
Assert.expectEq( "ns = new Namespace('duh'), QName(ns, 'name')",
    "duh::name",
    (ns = new Namespace('duh'), QName(ns, 'name').toString() ));
    
Assert.expectEq( "ns = new Namespace('duh'), typeof QName(ns, 'name')",
    "object",
    ( ns = new Namespace('duh'), typeof QName(ns, 'name') ));

Assert.expectEq( "ns = new Namespace('duh'), typeof QName(ns, 'name')",
    true,
    ( ns = new Namespace('duh'), QName(ns, 'name') instanceof QName));


END();
