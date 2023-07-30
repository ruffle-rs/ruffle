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

START("13.2.1 - Namespace Constructor as Function");




function convertToString(o:Object){
  return o.toString();
}

n = Namespace();
m = new Namespace();
TEST(1, typeof(m), typeof(n));
TEST(2, m.prefix, n.prefix);
TEST(3, m.uri, n.uri);

n = Namespace("http://foobar/");
m = new Namespace("http://foobar/");
TEST(4, typeof(m), typeof(n));
TEST(5, m.prefix, n.prefix);
TEST(6, m.uri, n.uri);

n = Namespace("foobar", "http://foobar/");
m = new Namespace("foobar", "http://foobar/");
TEST(7, typeof(m), typeof(n));
TEST(8, m.prefix, n.prefix);
TEST(9, m.uri, n.uri);

n = Namespace(m);
TEST(10, m, n);

var thisXML = "<XML><TEAM>Giants</TEAM><CITY>San Francisco</CITY></XML>";
var NULL_OBJECT = "";
 

// value is not supplied
Assert.expectEq( "Namespace()", NULL_OBJECT, Namespace().toString());
Assert.expectEq( "typeof Namespace()", "object", typeof Namespace() );
//Assert.expectEq( "Namespace().__proto__ == Namespace.prototype", true, Namespace().__proto__ == Namespace.prototype);

//One value is supplied
Assert.expectEq( "Namespace('pfx').toString()", 'pfx', Namespace('pfx').toString() );
Assert.expectEq( "typeof Namespace('pfx')", "object", typeof Namespace('pfx') );
//Assert.expectEq( "Namespace('pfx').__proto__ == Namespace.prototype", true, Namespace('pfx').__proto__ == Namespace.prototype);

var ns = new Namespace('http://foo.bar');
Assert.expectEq( "Namespace(nsObj).toString()", 'http://foo.bar', Namespace(ns).toString() );

//Two values are supplied
Assert.expectEq( "Namespace('pfx','ns') == new Namespace('pfx', 'ns')", (new Namespace('pfx', 'http://www.w3.org/TR/html4/')).toString(), (Namespace('pfx','http://www.w3.org/TR/html4/')).toString() );
Assert.expectEq( "typeof Namespace('pfx','http://www.w3.org/TR/html4/')", "object", typeof Namespace('pfx','http://www.w3.org/TR/html4/') );
//Assert.expectEq( "Namespace('pfx','http://www.w3.org/TR/html4/').__proto__ == Namespace.prototype", true, Namespace('pfx','http://www.w3.org/TR/html4/').__proto__ == Namespace.prototype );

ns = new Namespace('pfx', 'http://foo.bar');
Assert.expectEq( "Namespace(nsObj).toString()", 'http://foo.bar', Namespace(ns).toString() );

END();
